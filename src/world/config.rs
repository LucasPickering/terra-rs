use config::{Config, ConfigError, File};
use log::{debug, info};
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
        info!("Loading world config...");
        let config = WorldConfig {
            seed: 1292848,
            tile_radius: 16,
            elevation: NoiseFnConfig {
                octaves: 12,
                frequency: 1.0,
                lacunarity: 4.0,
                persistence: 0.5,
            },
            humidity: NoiseFnConfig {
                octaves: 3,
                frequency: 2.0,
                lacunarity: 2.0,
                persistence: 0.25,
            },
        };
        // let mut settings = Config::default();
        // // Look for "./terra.toml"
        // settings.merge(File::with_name("terra.toml"))?;
        // let config = settings.try_into::<Self>()?;
        debug!("Loaded world config:\n{:#?}", &config);
        Ok(config)
    }
}
