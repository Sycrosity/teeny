use embassy_net::Stack;
use esp_wifi::wifi::{
    AuthMethod, ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
    WifiStaDevice, WifiState,
};

use crate::prelude::*;

#[task]
pub async fn connection(mut controller: WifiController<'static>) {
    debug!("Start connection task");
    debug!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        if esp_wifi::wifi::get_wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await;
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap_or_default(),
                password: PASSWORD.try_into().unwrap_or_default(),
                auth_method: if PASSWORD.is_empty() {
                    AuthMethod::None
                } else {
                    AuthMethod::default()
                },
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("Starting wifi");
            controller.start().await.unwrap();
            info!("Wifi started!");
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
}

#[task]
pub async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await;
}
