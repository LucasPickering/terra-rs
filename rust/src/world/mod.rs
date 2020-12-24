mod generate;
pub mod hex;
pub mod tile;

use crate::{
    timed,
    util::{Color3, NumRange},
    world::{generate::WorldBuilder, hex::WorldMap, tile::Tile},
    WorldConfig,
};
use js_sys::Array;
use log::info;
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BiomeType {
    Water,
    Land,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Biome {
    // Water
    Ocean,
    Coast,
    Lake,

    // Land
    Snow,
    Desert,
    Alpine,
    Jungle,
    Forest,
    Plains,
}

impl Biome {
    pub fn biome_type(self) -> BiomeType {
        match self {
            Self::Ocean | Self::Coast | Self::Lake => BiomeType::Water,
            Self::Snow
            | Self::Desert
            | Self::Alpine
            | Self::Jungle
            | Self::Forest
            | Self::Plains => BiomeType::Land,
        }
    }

    pub fn color(self) -> Color3 {
        match self {
            Self::Ocean => Color3::new(0.08, 0.30, 0.64).unwrap(),
            Self::Coast => Color3::new_int(32, 166, 178),
            Self::Lake => Color3::new_int(72, 192, 240),

            Self::Snow => Color3::new(0.75, 0.75, 0.75).unwrap(),
            Self::Desert => Color3::new(0.84, 0.80, 0.42).unwrap(),
            Self::Alpine => Color3::new(0.39, 0.48, 0.37).unwrap(),
            Self::Jungle => Color3::new(0.17, 0.70, 0.12).unwrap(),
            Self::Forest => Color3::new(0.09, 0.48, 0.0).unwrap(),
            Self::Plains => Color3::new(0.68, 0.79, 0.45).unwrap(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    tiles: WorldMap<Tile>,
}

impl World {
    /// All tiles above this elevation are guaranteed to be non-ocean. All tiles
    /// at OR below _could_ be ocean, but the actual chance depends upon the
    /// ocean generation logic.
    pub const SEA_LEVEL: f64 = 0.0;
    pub const ELEVATION_RANGE: NumRange<f64> = NumRange::new(-100.0, 100.0);
    pub const HUMIDITY_RANGE: NumRange<f64> = NumRange::new(0.0, 1.0);

    pub fn tiles(&self) -> &WorldMap<Tile> {
        &self.tiles
    }

    pub fn generate(config: WorldConfig) -> Self {
        info!("Generating world");
        let tiles = timed!(
            "World generation",
            WorldBuilder::new(config).generate_world()
        );
        Self { config, tiles }
    }
}

#[wasm_bindgen]
impl World {
    /// A type-hacked accessor to get all tiles in the world for Wasm. This
    /// typing can be cleaned up after https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen]
    pub fn tiles_array(&self) -> TileArray {
        self.tiles
            .iter()
            .copied()
            .map(JsValue::from)
            .collect::<Array>()
            .unchecked_into()
    }
}

// Types that we can't natively return. These are assigned TS types, but
// these types aren't actually verified by the compiler. Be careful
// here!
#[wasm_bindgen]
extern "C" {

    /// Type hack needed until https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen(typescript_type = "Tile[]")]
    pub type TileArray;
}
