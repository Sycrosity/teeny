#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Config, Stack, StackResources,
};
use embassy_time::Ticker;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::ClockControl,
    gpio::{AnyInput, Io, Level, Pull},
    peripherals::{Peripherals, ADC1},
    rng::Rng,
    sha::{Sha, ShaMode},
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::wifi::WifiStaDevice;
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::{Method, RequestBuilder},
};
use teeny::{
    blink::blink,
    buttons::{
        display_play_pause, display_skip, publish_play_pause, publish_raw_skip, publish_skip,
    },
    display::{display_shapes, screen_counter},
    net::{connection, net_task},
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

    // #[cfg(feature = "alloc")]
    // spotify_mini::alloc::init_heap();

    let peripherals = Peripherals::take();

    let system = esp_hal::system::SystemControl::new(peripherals.SYSTEM);

    let io: Io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);

    esp_hal_embassy::init(&clocks, timer_group0);

    let mut adc1_config = AdcConfig::new();

    let rng = Rng::new(peripherals.RNG);
    let mut rng = *RNG.init(rng);

    #[cfg(target_arch = "riscv32")]
    let _hasher = Sha::new(peripherals.SHA, ShaMode::SHA256, None);
    #[cfg(target_arch = "xtensa")]
    let _hasher = Sha::new(peripherals.SHA, ShaMode::SHA256);

    #[cfg(target_arch = "xtensa")]
    let pot_pin = adc1_config.enable_pin(io.pins.gpio32, Attenuation::Attenuation11dB);
    #[cfg(target_arch = "riscv32")]
    let pot_pin = adc1_config.enable_pin_with_cal::<_, esp_hal::analog::adc::AdcCalCurve<ADC1>>(
        io.pins.gpio3,
        Attenuation::Attenuation11dB,
    );

    let adc1 =
        &*SHARED_ADC.init_with(|| Mutex::new(Adc::<ADC1>::new(peripherals.ADC1, adc1_config)));

    #[cfg(target_arch = "xtensa")]
    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, None).timer0;
    #[cfg(target_arch = "riscv32")]
    let timer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;

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

    spawner.spawn(blink(internal_led)).ok();
    spawner.must_spawn(publish_play_pause(play_pause_button));
    spawner.must_spawn(publish_raw_skip(skip_button));
    spawner.must_spawn(publish_skip());
    spawner.must_spawn(publish_volume(adc1, pot_pin));

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let config = Config::dhcpv4(Default::default());

    let seed = ((rng.random() as u64) << u32::BITS) + rng.random() as u64;

    // Init network stack
    let stack = make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    spawner.must_spawn(connection(controller));
    spawner.must_spawn(net_task(stack));

    loop {
        trace!("Checking stack state...");
        if stack.is_link_up() {
            debug!("Link is up!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address... ");
    loop {
        if let Some(config) = stack.config_v4() {
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

    loop {
        info!("Starting wifi loop...");

        let mut rx_buf = [0; 16640];
        let mut tx_buf = [0; 16640];

        let state = TcpClientState::<1, 4096, 4096>::new();

        let tcp_client = TcpClient::new(stack, &state);

        let dns_socket = DnsSocket::new(stack);

        let config = TlsConfig::new(seed, &mut rx_buf, &mut tx_buf, TlsVerify::None);

        let mut client = HttpClient::new_with_tls(&tcp_client, &dns_socket, config);

        debug!("HttpClient created");

        let headers = [
            ("user-agent", "teeny/0.1.0"),
            ("Host", "example.com"),
            ("accept", "application/json"),
            // ("connection", "close"),
        ];

        let mut header_buf = [0; 1024];

        let mut request = client
            .request(Method::GET, "https://example.com")
            .await
            .unwrap()
            .content_type(reqwless::headers::ContentType::TextPlain)
            .headers(&headers);

        debug!("Request sent");

        let response = request.send(&mut header_buf).await.unwrap();

        let content_len = response.content_length.unwrap();

        debug!("Response Recieved");

        let mut buf = [0; 50 * 1024];

        if let Err(e) = response.body().reader().read_to_end(&mut buf).await {
            error!("{e:?}");
            break;
        }

        print!("{:#?}", core::str::from_utf8(&buf[..content_len]).unwrap());

        Timer::after(Duration::from_secs(3)).await;
    }

    let mut ticker = Ticker::every(Duration::from_millis(1000));

    loop {
        trace!("KeepAlive tick");
        ticker.next().await;
    }
}
