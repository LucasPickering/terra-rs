use crate::input::InputBindings;
use log::info;
use serde::Deserialize;

// hack for loading config. At some point we probably will want to pull it from
// local storage or smth but this works for now
const CONFIG_JSON_STR: &str = include_str!("./terra.json");

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
    #[serde(default)]
    pub seed: u32,
    pub tile_radius: usize,
    pub elevation: NoiseFnConfig,
    pub humidity: NoiseFnConfig,
}

/// Configuration for an instance of Terra. Defines world gen, input bindings,
/// etc.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct InputConfig {
    /// Configurable keybinds
    pub bindings: InputBindings,
}

/// Configuration for an instance of Terra. Defines world gen, input bindings,
/// etc.
#[derive(Clone, Debug, Deserialize)]
pub struct TerraConfig {
    pub world: WorldConfig,
    pub input: InputConfig,
}

impl TerraConfig {
    /// Load config from a static file on the server
    pub async fn load() -> anyhow::Result<Self> {
        info!("Loading config");
        let config: TerraConfig = serde_json::from_str(CONFIG_JSON_STR)?;
        info!("Loaded config: {:#?}", &config);
        Ok(config)
    }
}
