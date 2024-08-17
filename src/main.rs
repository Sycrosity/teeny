#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsSocket,
    tcp::{
        client::{TcpClient, TcpClientState},
        TcpSocket,
    },
    Config, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfigV4,
};
use embassy_time::Ticker;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::ClockControl,
    gpio::{AnyInput, Io, Level, Pull},
    peripherals::{Peripherals, ADC1},
    rng::Rng,
    sha::{Sha, ShaMode},
    timer::{OneShotTimer, PeriodicTimer},
};
use esp_println::println;
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::{Method, RequestBuilder},
};
use static_cell::make_static;
use teeny::{
    auth::AuthParams,
    blink::blink,
    buttons::{
        display_play_pause, display_skip, publish_play_pause, publish_raw_skip, publish_skip,
    },
    display::{display_shapes, screen_counter},
    net::{self, ap_task, connection, random_utf8, wifi_task},
    prelude::*,
    volume::{display_volume, publish_volume},
};

#[main]
async fn main(spawner: Spawner) -> ! {
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/
    // rust-lang/cargo/issues/10358
    #[cfg(feature = "log")]
    teeny::logger::init_logger_from_env();
    info!("Logger is setup");
    println!("Hello world!");

    #[cfg(feature = "alloc")]
    teeny::alloc::init_heap();

    let peripherals = Peripherals::take();

    let system = esp_hal::system::SystemControl::new(peripherals.SYSTEM);

    let io: Io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();

    #[cfg(feature = "esp32")]
    {
        let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1, &clocks, None);
        esp_hal_embassy::init(
            &clocks,
            make_static!([OneShotTimer::new(timg1.timer0.into())]),
        );
    }

    #[cfg(not(feature = "esp32"))]
    {
        let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
        esp_hal_embassy::init(
            &clocks,
            make_static!([OneShotTimer::new(systimer.alarm0.into())]),
        );
    }

    let mut adc1_config = AdcConfig::new();

    let rng = Rng::new(peripherals.RNG);
    let mut rng = *RNG.init(rng);

    #[cfg(target_arch = "riscv32")]
    let mut sha = Sha::new(peripherals.SHA, ShaMode::SHA256, None);
    #[cfg(target_arch = "xtensa")]
    let mut sha = Sha::new(peripherals.SHA, ShaMode::SHA256);

    #[cfg(target_arch = "xtensa")]
    let pot_pin = adc1_config.enable_pin(io.pins.gpio32, Attenuation::Attenuation11dB);
    #[cfg(target_arch = "riscv32")]
    let pot_pin = adc1_config.enable_pin_with_cal::<_, esp_hal::analog::adc::AdcCalCurve<ADC1>>(
        io.pins.gpio3,
        Attenuation::Attenuation11dB,
    );

    let adc1 =
        &*SHARED_ADC.init_with(|| Mutex::new(Adc::<ADC1>::new(peripherals.ADC1, adc1_config)));

    // #[cfg(target_arch = "xtensa")]
    // let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    // #[cfg(target_arch = "riscv32")]
    // let timer =
    // esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;

    let timer = PeriodicTimer::new(
        esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0, &clocks, None)
            .timer0
            .into(),
    );

    let init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timer,
        rng,
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let internal_led = if cfg!(feature = "esp32") {
        AnyOutput::new(io.pins.gpio2, Level::Low)
    } else if cfg!(feature = "esp32c3") {
        AnyOutput::new(io.pins.gpio8, Level::Low)
    } else {
        unreachable!("Unsupported chip")
    };

    let scl = io.pins.gpio7;
    let sda = io.pins.gpio6;

    let play_pause_button = AnyInput::new(io.pins.gpio10, Pull::Down);
    let skip_button = AnyInput::new(io.pins.gpio5, Pull::Down);

    let i2c_bus = I2C_BUS.init_with(|| {
        Mutex::new(I2C::new_async(
            peripherals.I2C0,
            sda,
            scl,
            400u32.kHz(),
            &clocks,
        ))
    });

    let wifi = peripherals.WIFI;
    let (ap_interface, wifi_interface, controller) =
        esp_wifi::wifi::new_ap_sta(&init, wifi).unwrap();

    let ap_config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 2, 1), 24),
        gateway: Some(Ipv4Address::from_bytes(&[192, 168, 2, 1])),
        dns_servers: Default::default(),
    });

    let wifi_config = Config::dhcpv4(Default::default());

    let seed = ((rng.random() as u64) << u32::BITS) + rng.random() as u64;

    // Init access point stack
    let ap_stack = make_static!(Stack::new(
        ap_interface,
        ap_config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    // Init wifi networking stack
    let wifi_stack = make_static!(Stack::new(
        wifi_interface,
        wifi_config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    spawner.must_spawn(connection(controller, rng));
    spawner.must_spawn(ap_task(ap_stack));
    spawner.must_spawn(wifi_task(wifi_stack));

    loop {
        trace!("Checking stack state...");
        if wifi_stack.is_link_up() {
            debug!("Link is up!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    loop {
        if ap_stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Connect to the AP `esp-wifi` and point your browser to http://192.168.2.1:8080/");
    println!("Use a static IP in the range 192.168.2.2 .. 192.168.2.255, use gateway 192.168.2.1");

    info!("Waiting to get IP address... ");
    loop {
        if let Some(config) = wifi_stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    spawner.spawn(screen_counter(I2cDevice::new(i2c_bus))).ok();
    spawner.spawn(display_shapes(I2cDevice::new(i2c_bus))).ok();
    spawner.spawn(display_volume(I2cDevice::new(i2c_bus))).ok();
    spawner.spawn(display_skip(I2cDevice::new(i2c_bus))).ok();
    spawner
        .spawn(display_play_pause(I2cDevice::new(i2c_bus)))
        .ok();

    let mut ap_rx_buffer = [0; 1536];
    let mut ap_tx_buffer = [0; 1536];

    let mut ap_socket = TcpSocket::new(ap_stack, &mut ap_rx_buffer, &mut ap_tx_buffer);

    ap_socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    let mut wifi_rx_buf = [0; 16640];
    let mut wifi_tx_buf = [0; 16640];

    let state = TcpClientState::<1, 4096, 4096>::new();

    let tcp_client = TcpClient::new(wifi_stack, &state);

    let dns_socket = DnsSocket::new(wifi_stack);

    let config = TlsConfig::new(seed, &mut wifi_rx_buf, &mut wifi_tx_buf, TlsVerify::None);

    let mut client = HttpClient::new_with_tls(&tcp_client, &dns_socket, config);

    debug!("Http Client created");

    loop {
        info!("Starting wifi loop...");

        let code_challenge_raw = random_utf8::<64>(rng);

        // convert the vec to a string
        // SAFETY: the verifier is guaranteed to be valid utf8 as it is base64 encoded
        let code_challenge: String<64> = String::from_utf8(
            Vec::from_slice(&code_challenge_raw)
                .expect("should be enough bytes for cloning into the vec"),
        )
        .expect("Base64 encoding is valid utf8");

        let token = "TOKEN_GOES_HERE";

        let mut string: String<64> = String::new();

        string.push_str("Bearer ").unwrap();
        string.push_str(token).unwrap();
        let mut code_challenge_raw = code_challenge_raw.as_slice();

        // SHA256 Hashing the verifier
        while !code_challenge_raw.is_empty() {
            // SAFETY: All the HW Sha functions are infallible so unwrap is fine to use if
            // you use block!
            code_challenge_raw = block!(sha.update(code_challenge_raw)).unwrap();
        }

        let mut hash_buffer: [u8; 32] = [0; 32];

        block!(sha.finish(hash_buffer.as_mut_slice())).unwrap();

        let mut base64_buf: Vec<u8, 47> = Vec::new();
        base64_buf.resize(32 * 4 / 3 + 4, 0).unwrap();

        // Encode the hash to base64
        let real_b64_len = BASE64_STANDARD_NO_PAD
            .encode_slice(hash_buffer, &mut base64_buf)
            .unwrap();

        base64_buf.truncate(real_b64_len);

        let _verifier_hash = String::from_utf8(base64_buf).unwrap();

        let auth_params = AuthParams {
            response_type: "code",
            client_id: CLIENT_ID,
            scope: "user-read-private%20user-read-email",
            code_challenge,
            code_challenge_method: "S256",
            redirect_uri: "http://192.168.2.1/callback",
        };

        let headers = [
            ("User-Agent", "teeny/0.1.0"),
            ("Accept", "*/*"),
            ("Connection", "close"),
        ];

        let mut header_buf = [0; 1024 * 8];

        let mut auth_path: String<512> = String::new();

        auth_path.push_str("/authorize").unwrap();
        auth_path
            .push_str(auth_params.to_string().as_str())
            .inspect_err(|e| println!("{e:?}"))
            .unwrap();

        println!("{:#?}", &auth_path);

        let mut request = client
            .request(Method::GET, "https://accounts.spotify.com")
            .await
            .unwrap()
            .path(&auth_path)
            .headers(&headers);

        // println!("{:#?}", &request.build());

        let response = request.send(&mut header_buf).await.unwrap();

        debug!("Request sent");

        let content_len = response.content_length.unwrap();

        debug!("Response Recieved");

        let mut buf = [0; 32 * 1024];

        if let Err(e) = response.body().reader().read_to_end(&mut buf).await {
            error!("Error: {e:?}");
            break;
        }

        println!("{:#?}", core::str::from_utf8(&buf[..content_len]).unwrap());

        {

            // let token = "TOKEN_GOES_HERE";
            // let mut string: String<64> = String::new();
            // string.push_str("Bearer ").unwrap();
            // string.push_str(token).unwrap();

            // let headers = [
            //     ("User-Agent", "teeny/0.1.0"),
            //     ("Accept", "*/*"),
            //     ("Connection", "close"),
            //     ("Authorization", string.as_str()),
            // ];

            // let mut request = client
            //     .request(Method::GET, "https://accounts.spotify.com")
            //     .await
            //     .unwrap()
            //     .path("/authorise")
            //     .headers(&headers);

            // let response = request.send(&mut header_buf).await.unwrap();

            // debug!("Request sent");

            // let content_len = response.content_length.unwrap();

            // debug!("Response Recieved");

            // let mut buf = [0; 50 * 1024];

            // if let Err(e) = response.body().reader().read_to_end(&mut
            // buf).await {     error!("Error: {e:?}");
            //     break;
            // }

            // println!("{:#?}",
            // core::str::from_utf8(&buf[..content_len]).unwrap());
        }

        Timer::after(Duration::from_secs(3)).await;
    }

    let mut ticker = Ticker::every(Duration::from_millis(1000));

    loop {
        trace!("KeepAlive tick");
        ticker.next().await;
    }
}
