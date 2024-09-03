#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![allow(unused)]

use core::str;

// use data::{DhcpLease, SpotifyAccessToken, SpotifyData};
use dhcp::{dhcp, ROUTER_IP};
use embassy_executor::Spawner;
use embassy_net::{Config, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embedded_storage::nor_flash::MultiwriteNorFlash;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::ClockControl,
    gpio::{AnyInput, Io, Level, Pull},
    peripherals::{Peripherals, ADC1},
    rng::Rng,
    sha::Sha256,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_storage::FlashStorage;
use esp_wifi::wifi::WifiApDevice;
use serde::{Deserialize, Serialize};
use teeny::{
    blink::blink,
    // data::TeenyData,
    net::{self, ap_task, connection, AppRouter, GlobalState, WifiCredentials},
    prelude::*,
};

#[main]
async fn main(spawner: Spawner) {
    //MARK: setup

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
        let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks);

        esp_hal_embassy::init(&clocks, timg1.timer0);
    }

    #[cfg(not(feature = "esp32"))]
    {
        let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER)
            .split::<esp_hal::timer::systimer::Target>();

        esp_hal_embassy::init(&clocks, systimer.alarm0);
    }

    let mut adc1_config = AdcConfig::new();

    let rng = Rng::new(peripherals.RNG);
    let mut rng = *RNG.init(rng);

    let sha256 = Sha256::new();
    // Sha::update(&mut sha1, remaining).unwrap();

    #[cfg(target_arch = "xtensa")]
    let pot_pin = adc1_config.enable_pin(io.pins.gpio32, Attenuation::Attenuation11dB);
    #[cfg(target_arch = "riscv32")]
    let pot_pin = adc1_config.enable_pin_with_cal::<_, esp_hal::analog::adc::AdcCalCurve<ADC1>>(
        io.pins.gpio3,
        Attenuation::Attenuation11dB,
    );

    let adc1 =
        &*SHARED_ADC.init_with(|| Mutex::new(Adc::<ADC1>::new(peripherals.ADC1, adc1_config)));

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);

    let init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timg0.timer0,
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
    let (ap_interface, sta_interface, controller) =
        esp_wifi::wifi::new_ap_sta(&init, wifi).unwrap();

    let ap_config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(ROUTER_IP, 24),
        gateway: Some(ROUTER_IP),
        dns_servers: Default::default(),
    });

    // let dhcp_config = DhcpConfig::default();

    // let socket = smoltcp::socket::dhcpv4::Socket::new();
    // let handle = _s.sockets.add(socket);
    // self.dhcp_socket = Some(handle);

    let wifi_config = Config::dhcpv4(Default::default());

    let seed = ((rng.random() as u64) << u32::BITS) + rng.random() as u64;

    // Init access point stack
    let ap_stack = mk_static!(
        Stack<esp_wifi::wifi::WifiDevice<'_, WifiApDevice>>,
        Stack::new(
            ap_interface,
            ap_config,
            // make_static!(StackResources::<5>::new()),
            mk_static!(
                StackResources::<{ net::WEB_TASK_POOL_SIZE + 2 }>,
                StackResources::<{ net::WEB_TASK_POOL_SIZE + 2 }>::new()
            ),
            seed
        )
    );

    // // Init wifi networking stack
    // let wifi_stack = make_static!(Stack::new(
    //     wifi_interface,
    //     wifi_config,
    //     make_static!(StackResources::<3>::new()),
    //     seed
    // ));

    spawner.spawn(blink(internal_led)).ok();
    // spawner.must_spawn(publish_play_pause(play_pause_button));
    // spawner.must_spawn(publish_raw_skip(skip_button));
    // spawner.must_spawn(publish_skip());
    // spawner.must_spawn(publish_volume(adc1, pot_pin));

    // spawner.spawn(screen_counter(I2cDevice::new(i2c_bus))).ok();
    // spawner.spawn(display_shapes(I2cDevice::new(i2c_bus))).ok();
    // spawner.spawn(display_volume(I2cDevice::new(i2c_bus))).ok();
    // spawner.spawn(display_skip(I2cDevice::new(i2c_bus))).ok();
    // spawner
    //     .spawn(display_play_pause(I2cDevice::new(i2c_bus)))
    //     .ok();

    spawner.must_spawn(connection(controller, rng));
    spawner.must_spawn(ap_task(ap_stack));
    // spawner.must_spawn(wifi_task(wifi_stack));

    // loop {
    //     trace!("Checking stack state...");
    //     if wifi_stack.is_link_up() {
    //         debug!("Link is up!");
    //         break;
    //     }
    //     Timer::after(Duration::from_millis(500)).await;
    // }

    loop {
        if ap_stack.is_link_up() {
            break;
        }
        trace!("AP link is not up");
        Timer::after(Duration::from_millis(500)).await;
    }

    spawner.must_spawn(dhcp(ap_stack));

    //MARK: picoserve
    let app = mk_static!(picoserve::Router<AppRouter,GlobalState>, net::app_router());

    let config = mk_static!(
        picoserve::Config<Duration>,
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(1)),
            write: Some(Duration::from_secs(1)),
        })
        .keep_connection_alive()
    );

    let wifi_creds = net::WifiCredentialsState(
        mk_static!(Mutex<CriticalSectionRawMutex, WifiCredentials>, Mutex::new(WifiCredentials::default())),
    );

    //run picoserve

    for id in 0..net::WEB_TASK_POOL_SIZE {
        // let mut rand_bytes: [u8; 16] = [0; 16];
        // rng.read(&mut rand_bytes);

        spawner.must_spawn(net::site_task(
            id,
            ap_stack,
            app,
            config,
            GlobalState { wifi_creds },
        ));
    }
}

// pub struct DnsServer<'a, 's, 'n>
// where
//     'n: 's,
// {
//     dns_socket: &'a mut UdpSocket<'s, 'n, WifiApDevice>,
//     dns_buffer: [u8; 1536],
//     ip: [u8; 4],
//     ttl: Duration,
// }

// impl<'a, 's, 'n> DnsServer<'a, 's, 'n>
// where
//     'n: 's,
// {
//     fn new(dns_socket: &'a mut UdpSocket<'s, 'n, WifiApDevice>, ip: [u8; 4],
// ttl: Duration) -> Self {         Self {
//             dns_socket,
//             dns_buffer: [0u8; 1536],
//             ip,
//             ttl,
//         }
//     }

//     fn handle_dns(&mut self) {
//         self.dns_socket.work();

//         match self.dns_socket.receive(&mut self.dns_buffer) {
//             Ok((len, src_addr, src_port)) => {
//                 if len > 0 {
//                     log::info!("DNS FROM {:?} / {}", src_addr, src_port);
//                     log::info!("DNS {:02x?}", &self.dns_buffer[..len]);

//                     let request = &self.dns_buffer[..len];
//                     let response: Vec<u8, 512> = Vec::new();

//                     let message =
// domain::base::Message::from_octets(request).unwrap();
// log::info!("Processing message with header: {:?}", message.header());

//                     let mut responseb =
//
// domain::base::MessageBuilder::from_target(response).unwrap();

//                     let response = if matches!(message.header().opcode(),
// Opcode::QUERY) {                         log::info!("Message is of type
// Query, processing all questions");

//                         let mut answerb = responseb.start_answer(&message,
// Rcode::NOERROR).unwrap();

//                         for question in message.question() {
//                             let question = question.unwrap();

//                             if matches!(question.qtype(), Rtype::A) {
//                                 log::info!(
//                                     "Question {:?} is of type A, answering
// with IP {:?}, TTL {:?}",                                     question,
//                                     self.ip,
//                                     self.ttl
//                                 );

//                                 let record = Record::new(
//                                     question.qname(),
//                                     Class::In,
//                                     self.ttl.as_secs() as u32,
//                                     A::from_octets(self.ip[0], self.ip[1],
// self.ip[2], self.ip[3]),                                 );
//                                 log::info!("Answering question {:?} with
// {:?}", question, record);
// answerb.push(record).unwrap();                             } else {
//                                 log::info!(
//                                     "Question {:?} is not of type A, not
// answering",                                     question
//                                 );
//                             }
//                         }

//                         answerb.finish()
//                     } else {
//                         log::info!("Message is not of type Query, replying
// with NotImp");

//                         let headerb = responseb.header_mut();

//                         headerb.set_id(message.header().id());
//                         headerb.set_opcode(message.header().opcode());
//                         headerb.set_rd(message.header().rd());
//                         headerb.set_rcode(domain::base::iana::Rcode::NOTIMP);

//                         responseb.finish()
//                     };

//                     self.dns_socket.send(src_addr, src_port,
// &response).unwrap();                 }
//             }
//             _ => (),
//         }
//     }
// }

// const CONTENT: &[u8] = b"
// <!DOCTYPE html>
// <html lang=\"en\">
// <head>
//     <meta charset=\"UTF-8\">
//     <meta name=\"viewport\" content=\"width=device-width,
// initial-scale=1.0\">     <title>Hello World</title>
//     <style>
//         body {
//             display: flex;
//             justify-content: center;
//             align-items: center;
//             height: 100vh;
//             font-family: Arial, sans-serif;
//             background-color: #f2f2f2;
//         }

//         h1 {
//             font-size: 48px;
//             color: #333;
//             text-align: center;
//             text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.5);
//             animation: rainbow 5s linear infinite;
//         }

//         @keyframes rainbow {
//             0% { color: red; }
//             14% { color: orange; }
//             28% { color: yellow; }
//             42% { color: green; }
//             57% { color: blue; }
//             71% { color: indigo; }
//             85% { color: violet; }
//             100% { color: red; }
//         }
//     </style>
// </head>
// <body>
//     <h1>Hello World! Hello esp-wifi! Hello captive-portal!</h1>
// </body>
// </html>
// ";
