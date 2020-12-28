#![feature(const_fn)]

use crate::world::World;
use log::info;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

mod util;
mod world;

/// Config for a particular noise generation function
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct NoiseFnConfig {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
    pub exponent: f64,
}

/// Configuration that defines a world gen process. Two worlds generated with
/// same config will always be identical.
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WorldConfig {
    pub seed: u64,
    pub tile_radius: u16,
    /// Ratio of humidity:initial_runoff. For example, if the scale is 0.5,
    /// then a tile with 0.5 humidity will start with 0.25 m³ runoff on it.
    pub rainfall_scale: f64,
    pub elevation: NoiseFnConfig,
    pub humidity: NoiseFnConfig,
}

/// Top-level struct for a Terra instance. This holds every we need to render
/// and interact with Terra from the outside. All interaction to/from wasm
/// should go through this struct.
#[wasm_bindgen]
pub struct Terra;

#[wasm_bindgen]
impl Terra {
    /// Initialize global state needed for world generation. Should be called
    /// once per app instance.
    #[wasm_bindgen(constructor)]
    pub fn initialize() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::default());
        Self
    }

    /// Generate a new world with the given config.
    #[wasm_bindgen]
    pub fn generate_world(&self, config: JsValue) -> Result<World, JsValue> {
        info!("Loading config");
        let config: WorldConfig = serde_wasm_bindgen::from_value(config)?;
        info!("Loaded config: {:#?}", &config);

        Ok(World::generate(config))
    }
}
