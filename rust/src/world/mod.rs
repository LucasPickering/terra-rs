mod generate;
pub mod hex;
pub mod tile;

use crate::{
    config::WorldConfig,
    timed,
    util::Color3,
    world::{generate::WorldBuilder, tile::TileMap},
};
use log::info;

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
