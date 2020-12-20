mod generate;
pub mod hex;
pub mod tile;

use crate::{
    timed,
    util::Color3,
    world::{
        generate::WorldBuilder,
        hex::HasHexPosition,
        tile::{Tile, TileLens, TileMap},
    },
    WorldConfig,
};
use js_sys::Array;
use log::info;
use serde::Serialize;
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BiomeType {
    Water,
    Land,
}

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
    Beach,
    Cliff,
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
            | Self::Plains
            | Self::Beach
            | Self::Cliff => BiomeType::Land,
        }
    }

    pub fn color(self) -> Color3 {
        match self {
            Self::Ocean => Color3::new(0.08, 0.30, 0.64),
            Self::Coast => Color3::new(0.22, 0.55, 0.78),
            Self::Lake => Color3::new(0.04, 0.55, 0.75),

            Self::Snow => Color3::new(0.75, 0.75, 0.75),
            Self::Desert => Color3::new(0.84, 0.80, 0.42),
            Self::Alpine => Color3::new(0.39, 0.48, 0.37),
            Self::Jungle => Color3::new(0.17, 0.70, 0.12),
            Self::Forest => Color3::new(0.09, 0.48, 0.0),
            Self::Plains => Color3::new(0.68, 0.79, 0.45),
            Self::Beach => Color3::new(0.95, 0.94, 0.35),
            Self::Cliff => Color3::new(0.21, 0.20, 0.17),
        }
        .unwrap()
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    tiles: TileMap,
}

impl World {
    /// All tiles above this elevation are guaranteed to be non-ocean. All tiles
    /// at OR below _could_ be ocean, but the actual chance depends upon the
    /// ocean generation logic.
    pub const SEA_LEVEL: f64 = 0.0;

    pub fn tiles(&self) -> &TileMap {
        &self.tiles
    }

    pub fn generate(config: WorldConfig) -> Self {
        info!("Generating world");
        let (tiles, elapsed) =
            timed!(WorldBuilder::new(config).generate_world());
        info!("Generated world in {}ms", elapsed.as_millis());
        Self { config, tiles }
    }
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen]
    pub fn tiles_render_info(&self, lens: TileLens) -> TileArray {
        let tiles: Vec<TileRenderInfo> = self
            .tiles
            .values()
            .map(|tile| {
                let pos = tile.position();
                TileRenderInfo {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    height: Tile::ELEVATION_RANGE
                        .map(&Tile::ELEVATION_RANGE.zeroed(), tile.elevation()),
                    color: tile.color(lens),
                }
            })
            .collect();
        tiles
            .into_iter()
            .map(JsValue::from)
            .collect::<Array>()
            .unchecked_into()
    }
}

/// A simplified version of a tile, to be sent over the WASM boundary
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct TileRenderInfo {
    pub x: isize,
    pub y: isize,
    pub z: isize,
    pub height: f64,
    /// Color for a particular lens. The lens is pre-determined when this
    /// struct is created, so it should be passed from TS.
    pub color: Color3,
}

// Types that we can't natively return. These are assigned TS types, but
// these types aren't actually verified by the compiler. Be careful
// here!
#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(typescript_type = "TileRenderInfo[]")]
    pub type TileArray;
}
