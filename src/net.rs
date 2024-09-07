use base64::prelude::*;
use embassy_net::Stack;
use esp_wifi::wifi::{
    AccessPointConfiguration, AuthMethod, ClientConfiguration, Configuration, WifiApDevice,
    WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState,
};
use picoserve::{
    extract::State,
    response::{DebugValue, IntoResponse},
    routing::get_service,
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
        let config = Configuration::AccessPoint(
            // ClientConfiguration {
            //     ssid: SSID.try_into().unwrap_or_default(),
            //     password: PASSWORD.try_into().unwrap_or_default(),
            //     auth_method: {
            //         #[allow(clippy::const_is_empty)]
            //         if PASSWORD.is_empty() {
            //             AuthMethod::None
            //         } else {
            //             AuthMethod::default()
            //         }
            //     },
            //     ..Default::default()
            // },
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
        // controller.connect()
        info!("Wifi started!");
    }

    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::ApStarted => {
                // debug!("Access point is connected");

                if esp_wifi::wifi::get_wifi_state() == WifiState::StaConnected {
                    // wait until we're no longer connected.
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
                controller.wait_for_event(WifiEvent::ApStaconnected).await;
                info!("AP/STA connected!");
            }
            WifiState::ApStopped => {
                error!("Access point stopped.");
            }
            _ => return,
        }
        Timer::after(Duration::from_millis(5000)).await;
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

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct WifiCredentials {
    ssid: String<32>,
    password: String<64>,
}

impl WifiCredentials {
    pub fn new(ssid: String<32>, password: String<64>) -> Self {
        Self { ssid, password }
    }
}

impl From<WifiCredentials> for ClientConfiguration {
    fn from(value: WifiCredentials) -> Self {
        Self {
            ssid: value.ssid,
            bssid: None,
            auth_method: if value.password.is_empty() {
                AuthMethod::None
            } else {
                AuthMethod::WPA2WPA3Personal
            },
            password: value.password,
            channel: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct WifiCredentialsState(pub &'static Mutex<CriticalSectionRawMutex, WifiCredentials>);

pub struct GlobalState {
    pub wifi_creds: WifiCredentialsState,
}

impl picoserve::extract::FromRef<GlobalState> for WifiCredentialsState {
    fn from_ref(state: &GlobalState) -> Self {
        state.wifi_creds
    }
}

pub const WEB_TASK_POOL_SIZE: usize = 4;

pub type AppRouter = impl picoserve::routing::PathRouter<GlobalState>;

#[task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn site_task(
    id: usize,
    // uuid: Uuid,
    stack: &'static Stack<WifiDevice<'static, WifiApDevice>>,
    app: &'static picoserve::Router<AppRouter, GlobalState>,
    config: &'static picoserve::Config<Duration>,
    state: GlobalState,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve_with_state(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
        &state,
    )
    .await
}

struct NotFound;

impl picoserve::routing::PathRouterService<GlobalState> for NotFound {
    async fn call_request_handler_service<
        R: picoserve::io::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        &self,
        _state: &GlobalState,
        _path_parameters: (),
        path: picoserve::request::Path<'_>,
        request: picoserve::request::Request<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        (
            picoserve::response::StatusCode::NOT_FOUND,
            format_args!("{:?} not found\n", path.encoded()),
        )
            .write_to(request.body_connection.finalize().await?, response_writer)
            .await
    }
}

pub fn app_router() -> picoserve::Router<AppRouter, GlobalState> {
    picoserve::Router::from_service(NotFound)
        .route(
            "/",
            get_service(picoserve::response::File::html(include_str!(
                "../dist/index.html"
            ))),
        )
        .route(
            "/favicon.svg",
            get_service(picoserve::response::File::with_content_type(
                "text/plain; charset=utf-8",
                include_str!("../dist/favicon.svg").as_bytes(),
            )),
        )
        .nest("/api", api_router())

    // .route(
    //     "/app.js",
    //     get_service(picoserve::response::File::javascript(include_str!(
    //         "../dist/app.js"
    //     ))),
    // )
}

pub fn api_router(
) -> picoserve::Router<impl picoserve::routing::PathRouter<GlobalState>, GlobalState> {
    picoserve::Router::new()
        .route(
            ("/set_ssid", picoserve::routing::parse_path_segment()),
            picoserve::routing::get(
                |ssid: String<32>,
                 State(WifiCredentialsState(config)): State<WifiCredentialsState>| async move {
                    config.lock().await.ssid = ssid;

                    DebugValue("success!")
                },
            ),
        )
        .route(
            ("/set_ssid", picoserve::routing::parse_path_segment()),
            picoserve::routing::get(
                |password: String<64>,
                 State(WifiCredentialsState(config)): State<WifiCredentialsState>| async move {
                    config.lock().await.password = password;

                    DebugValue("success!")
                },
            ),
        )
}
