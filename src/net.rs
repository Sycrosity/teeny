use base64::prelude::*;
use embassy_net::Stack;
use esp_wifi::wifi::{
    AccessPointConfiguration, AuthMethod, ClientConfiguration, Configuration, WifiApDevice,
    WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState,
};

use crate::prelude::*;

#[inline]
///Uses base64 to generate a valid random utf8 string from a random buffer
pub fn random_utf8<const LEN: usize>(mut rng: Rng) -> Vec<u8, LEN> {
    let mut input_buf: Vec<u8, LEN> = Vec::new();

    input_buf
        .resize((LEN / 4) * 3, 0)
        .expect("will be shorter than length of the vec");

    //read the number of bytes required such that the base64 output is `LEN` bytes
    // long.
    rng.read(&mut input_buf[..{ (LEN / 4) * 3 }]);

    let mut output_buf: Vec<u8, LEN> = Vec::new();

    //resize the output buffer to be able to hold the base64 encoded output
    output_buf.resize(LEN, 0).unwrap();

    println!("{}", input_buf.len());

    // write the base64 encoded verifier to a vec and get the length of the written
    // bytes - will be 4 less than the buffer length
    let base64_encoded_length = BASE64_URL_SAFE_NO_PAD
        .encode_slice(input_buf, &mut output_buf)
        .unwrap();

    output_buf.truncate(base64_encoded_length);

    output_buf
}

#[task]
pub async fn connection(mut controller: WifiController<'static>, _rng: Rng) {
    debug!("Start connection task");
    debug!("Device capabilities: {:?}", controller.get_capabilities());

    if !matches!(controller.is_started(), Ok(true)) {
        let config = Configuration::Mixed(
            ClientConfiguration {
                ssid: SSID.try_into().unwrap_or_default(),
                password: PASSWORD.try_into().unwrap_or_default(),
                auth_method: if PASSWORD.is_empty() {
                    AuthMethod::None
                } else {
                    AuthMethod::default()
                },
                ..Default::default()
            },
            AccessPointConfiguration {
                ssid: String::try_from("Teeny").expect("should be a valid access point SSID"),
                auth_method: AuthMethod::None,
                // password: {

                // let password: String<64> = String::from_utf8(
                //     Vec::from_slice(&random_utf8::<12>(rng)).expect("64 is larger than 10"),
                // )
                // .expect("10 is less than 64");

                // info!("Wifi Password: {:?}", &password);

                // info!("Wifi Password: {:?}", &password.as_bytes());
                // password
                // },
                ..Default::default()
            },
        );
        controller.set_configuration(&config).unwrap();
        info!("Starting wifi");
        controller.start().await.unwrap();
        info!("Wifi started!");
    }

    loop {
        match esp_wifi::wifi::get_ap_state() {
            WifiState::ApStarted => {
                println!("About to start access point...");

                if esp_wifi::wifi::get_wifi_state() == WifiState::StaConnected {
                    // wait until we're no longer connected
                    controller.wait_for_event(WifiEvent::StaDisconnected).await;
                    Timer::after(Duration::from_millis(5000)).await;
                }

                debug!("About to connect...");
                match controller.connect().await {
                    Ok(()) => info!("Wifi connected!"),
                    Err(e) => {
                        error!("Failed to connect to wifi: {e:?}");
                        Timer::after(Duration::from_millis(5000)).await;
                    }
                }
            }
            _ => return,
        }
    }
}

#[task]
pub async fn ap_task(stack: &'static Stack<WifiDevice<'static, WifiApDevice>>) {
    stack.run().await
}

#[task]
pub async fn wifi_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
