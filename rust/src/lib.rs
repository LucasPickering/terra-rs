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
    /// Number of different frequencies to add together. We can use multiple
    /// octaves to build a set of curves, then add them together to get our
    /// final function.
    pub octaves: usize,
    /// The frequency of the first (lowest) octave.
    pub frequency: f64,
    /// Constant to add to the frequency for each octave. E.g. if we have 3
    /// octaves, a base frequency of 1.0, and a lacunarity of 2.0, then our
    /// 3 octaves will be at [1.0, 3.0, 5.0].
    pub lacunarity: f64,
    /// Amplification factor for each octave. The first amplitude is always
    /// 1.0, then is multiplied by the persistence for each octave. E.g. with 3
    /// octaves and a persistence of 0.5, your amplitudes will be `[1.0, 0.5,
    /// 0.25]`.
    pub persistence: f64,
    /// Exponent to apply to values after generation. This is applied to
    /// normalized composite values. "Normalized" means they're in the range
    /// [0,1] (meaning we can apply any exponent and the values remain in that
    /// range) and "composite" means this is *after* we add all our octaves
    /// together.
    pub exponent: f64,
}

/// Configuration that defines a world gen process. Two worlds generated with
/// same config will always be identical.
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WorldConfig {
    /// RNG seed to use for all world gen. Two worlds generated with the same
    /// seed will always be identical.
    pub seed: u64,

    /// Distance from the center of the world to the edge (in tiles).
    pub radius: u16,

    /// Buffer space at the edge of the world where we gradually push
    /// elevations down, to ensure that the very edge of the world is all
    /// ocean. This is included **as part of the radius**, and therefore must
    /// be less than the radius.
    pub edge_buffer_size: u16,

    /// Exponent to apply to the function that pushes down elevations in the
    /// buffer zone. An exponent of 1.0 will push them linearly. Sub-1.0
    /// exponents will have a smooth dropoff closer to the middle, then get
    /// steeper towards the edge. Super-1.0 exponents will do the opposite
    /// (steep at first, then smooth out at the edge).
    pub edge_buffer_exponent: f64,

    /// Ratio of humidity:initial_runoff. For example, if the scale is 0.5,
    /// then a tile with 0.5 humidity will start with 0.25 m³ runoff on it.
    pub rainfall_scale: f64,

    /// Config for the noise function used to generate elevation values
    pub elevation: NoiseFnConfig,

    /// Config for the noise function used to generate humidity values
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
