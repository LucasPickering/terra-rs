use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct NoiseFnConfig {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct WorldConfig {
    pub seed: u32,
    pub tile_radius: usize,
    pub elevation: NoiseFnConfig,
    pub humidity: NoiseFnConfig,
}

impl WorldConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut settings = Config::default();
        // Look for "./terra.toml"
        settings.merge(File::with_name("terra.toml"))?;
        settings.try_into::<Self>()
    }
}
