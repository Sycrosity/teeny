#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::{raw::Executor, Spawner};
use embassy_net::{tcp::TcpSocket, Config, Ipv4Address, Stack, StackResources};
use embassy_time::Ticker;
use esp_hal::{
    analog::adc::{AdcCalCurve, AdcConfig, Attenuation, ADC},
    clock::ClockControl,
    gpio::IO,
    peripherals::{Peripherals, ADC1},
    rng::Rng,
    timer::TimerGroup,
};
use esp_println::println;
use esp_wifi::wifi::WifiStaDevice;
use httparse::{Header, Status};
use spotify_mini::{
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
    spotify_mini::logger::init_logger_from_env();
    info!("Logger is setup");
    println!("Hello world!");

    #[cfg(feature = "alloc")]
    spotify_mini::alloc::init_heap();

    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();

    let io: IO = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0);

    let mut adc1_config = AdcConfig::new();

    let rng = Rng::new(peripherals.RNG);
    let mut rng = *RNG.init(rng);

    let pot_pin = io.pins.gpio3.into_analog();
    let pot_pin = adc1_config
        .enable_pin_with_cal::<_, AdcCalCurve<ADC1>>(pot_pin, Attenuation::Attenuation11dB);

    let adc1 =
        &*SHARED_ADC.init_with(|| Mutex::new(ADC::<ADC1>::new(peripherals.ADC1, adc1_config)));

    let timer = esp_hal::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timer,
        rng,
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let internal_led = io.pins.gpio8.into_push_pull_output();

    let scl = io.pins.gpio7;
    let sda = io.pins.gpio6;

    let play_pause_button = io.pins.gpio10.into_pull_down_input();
    let skip_button = io.pins.gpio5.into_pull_down_input();

    let i2c_bus = I2C_BUS.init_with(|| {
        Mutex::new(I2C::new_async(
            peripherals.I2C0,
            sda,
            scl,
            400u32.kHz(),
            &clocks,
        ))
    });

    spawner.spawn(blink(internal_led.into())).ok();
    spawner.must_spawn(publish_play_pause(play_pause_button.into()));
    spawner.must_spawn(publish_raw_skip(skip_button.into()));
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

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        trace!("Checking stack state...");
        if stack.is_link_up() {
            debug!("Link is up!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");
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

    'wifi: loop {
        info!("Starting wifi loop...");

        Timer::after(Duration::from_millis(1_000)).await;

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(Duration::from_secs(10)));

        let address = Ipv4Address::new(142, 250, 185, 115);
        let port = 80;

        // let address = Ipv4Address::new(192, 168, 1, 44);
        // let port = 8000;
        let remote_endpoint = (address, port);
        info!("Connecting to \"{address}:{port}\"...");
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            error!("connect error: {:?}", e);
            continue;
        }
        info!("connected!");
        let mut buf = [0; 1024];

        let mut new_packet = true;

        let mut remaining_http_bytes = usize::MAX;

        'packet: loop {
            use embedded_io_async::Write;

            if remaining_http_bytes == 0 {
                debug!("HTTP Packet over");
                break 'wifi;
            }

            debug!("Starting new packet");

            if new_packet {
                let r = socket
                    .write_all(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
                    // .write_all(b"GET / HTTP/1.1\r\nHost: 192.168.1.44:8000\r\nAccept:
                    // */*\r\n\r\n")
                    .await;
                if let Err(e) = r {
                    warn!("write error: {:?}", e);
                    break 'packet;
                }
            }

            let socket_length = match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("read EOF");
                    break 'packet;
                }
                Ok(n) => n,
                Err(e) => {
                    if remaining_http_bytes != 0 {
                        warn!("read error: {:?}", e);
                    }
                    break 'packet;
                }
            };

            let header_offset = if new_packet {
                new_packet = false;

                let mut headers = [httparse::EMPTY_HEADER; 16];
                let mut res = httparse::Response::new(&mut headers);

                let offset = loop {
                    match res.parse(&buf[..socket_length]) {
                        Ok(status) => match status {
                            Status::Complete(offset) => break offset,
                            Status::Partial => warn!("Partial HTTP header recieved. Retrying..."),
                        },
                        Err(e) => {
                            error!("{}", e);
                            break 'packet;
                        }
                    };
                    Timer::after(Duration::from_secs(3)).await;
                };

                info!("{:#?}", &res.headers);

                res.headers.iter().for_each(|Header { name, value }| {
                    if *name == "Content-Length" {
                        if let Ok(content_length) =
                            str::parse::<usize>(unsafe { core::str::from_utf8_unchecked(value) })
                        {
                            remaining_http_bytes = content_length + offset;
                        } else {
                            warn!("Content-length could not be parsed!");
                        }
                    }
                });

                offset
            } else {
                0
            };

            info!("Remaining Bytes: {remaining_http_bytes:?}");

            info!(
                "Socket Length: {}, Header Offset: {}",
                socket_length, header_offset
            );

            remaining_http_bytes -= socket_length;

            println!(
                "{}",
                core::str::from_utf8(&buf[header_offset..socket_length]).unwrap()
            );
        }
        Timer::after(Duration::from_secs(3)).await;
    }

    let mut ticker = Ticker::every(Duration::from_millis(1000));

    loop {
        trace!("KeepAlive tick");

        // info!("Play Pause: {}", play_pause_pin.is_high());
        // warn!("Skip: {}", skip_pin.is_high());

        ticker.next().await;
    }
}
