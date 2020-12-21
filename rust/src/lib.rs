#![feature(const_fn)]
#![feature(map_first_last)]

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
}

/// Configuration that defines a world gen process. Two worlds generated with
/// same config will always be identical.
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WorldConfig {
    pub seed: u64,
    pub tile_radius: usize,
    pub elevation: NoiseFnConfig,
    pub humidity: NoiseFnConfig,
}

impl WorldConfig {
    /// Get the seed as a u32 value, which is needed for noise functions. This
    /// will take just the lower 32 bits of our seed.
    pub fn seed_u32(&self) -> u32 {
        (self.seed & 0xffffffff) as u32
    }
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
