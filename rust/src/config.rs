use crate::input::InputBindings;
use serde::Deserialize;

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
    // this causes a crash rn
    // #[serde(default = "rand::random")]
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

/// Configuration for an instance of Terra. Defines world gen, input bindings,
/// etc.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct InputConfig {
    /// Configurable keybinds
    pub bindings: InputBindings,
    /// Ratio between mouse movement and camera turn speed, in pixels/degree.
    pub mouse_sensitivity: f32,
    /// Vertical camera FOV
    pub fov: f32,
}

/// Configuration for an instance of Terra. Defines world gen, input bindings,
/// etc.
#[derive(Clone, Debug, Deserialize)]
pub struct TerraConfig {
    pub world: WorldConfig,
    pub input: InputConfig,
}
