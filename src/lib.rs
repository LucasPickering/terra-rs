use crate::{render::Scene, world::World};
use log::info;
use serde::Deserialize;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, Request, RequestInit, RequestMode, Response};

mod camera;
mod input;
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

impl WorldConfig {
    /// Load world config from a static file on the server
    async fn load() -> Result<Self, js_sys::Error> {
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

        let config: Self = json
            .into_serde()
            .map_err(|err| js_sys::Error::new(&err.to_string()))?;
        info!("Loaded config: {:#?}", &config);

        Ok(config)
    }
}

/// Top-level struct for a Terra instance. This holds every we need to render
/// Terra from the outside.
#[wasm_bindgen]
pub struct Terra {
    world: World,
    scene: Scene,
}

#[wasm_bindgen]
impl Terra {
    /// Initialize a Terra instance
    #[wasm_bindgen]
    pub async fn load(canvas_id: String) -> Result<Terra, js_sys::Error> {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::default());

        let config = WorldConfig::load().await?;
        let world = World::generate(config);
        let scene = Scene::new(&canvas_id, &world);

        Ok(Self { world, scene })
    }

    /// Run a single render. This is the entrypoint into the game loop, so it
    /// should be called once per frame.
    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.scene.render().unwrap_throw();
    }
}
