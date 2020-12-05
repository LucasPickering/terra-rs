use log::info;
use serde::Deserialize;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

mod render;
mod util;
mod world;

const CONFIG_URL: &str = "/terra.json";

/// Config for a particular noise generation function
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct NoiseFnConfig {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
}

/// Configuration that defines a world gen process. Two worlds generated with
/// same config will always be identical.
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WorldConfig {
    pub seed: u32,
    pub tile_radius: usize,
    pub elevation: NoiseFnConfig,
    pub humidity: NoiseFnConfig,
}

// TODO build our own error type that can convert from anyhow or js_sys::Error

/// Load world config from a static file on the server
async fn load_config() -> Result<WorldConfig, js_sys::Error> {
    info!("Loading config from {}", CONFIG_URL);
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(CONFIG_URL, &opts)?;
    request.headers().set("Accept", "application/json")?;

    let window = web_sys::window().unwrap();
    let resp_value =
        JsFuture::from(window.fetch_with_request(&request)).await?;

    // Read the response as JSON
    let resp: Response = resp_value.dyn_into().unwrap();
    let json = JsFuture::from(resp.json()?).await?;

    let config: WorldConfig = json
        .into_serde()
        .map_err(|err| js_sys::Error::new(&err.to_string()))?;
    info!("Loaded config: {:#?}", &config);

    Ok(config)
}

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), js_sys::Error> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());
    let config = load_config().await?;
    render::run(config);
    Ok(())
}
