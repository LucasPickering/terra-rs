mod elevation;

use crate::world::{
    Biome, HasHexPosition, HexPoint, HexPointMap, Tile, WorldConfig,
};
pub use elevation::*;
use log::info;
use std::fmt::Debug;

pub struct WorldBuilder<T> {
    config: WorldConfig,
    tiles: HexPointMap<T>,
}

impl WorldBuilder<()> {
    pub fn new(config: WorldConfig) -> Self {
        let mut tiles = HexPointMap::new();
        let radius: isize = config.tile_radius as isize;
        for x in -radius..=radius {
            for y in -radius..=radius {
                let pos = HexPoint::new(x, y, -(x + y));
                tiles.insert(pos, ());
            }
        }
        Self { config, tiles }
    }
}

impl<T> WorldBuilder<T> {
    /// A helper to run a generation step on this builder, returning a new
    /// builder. Allows for chained `.generate` calls.
    pub fn generate<U, G: Generate<T, U>>(
        self,
        generator: &G,
    ) -> WorldBuilder<U> {
        info!("Running generator {:?}", generator);
        let new_tiles = generator.generate(&self.config, self.tiles);
        WorldBuilder {
            config: self.config,
            tiles: new_tiles,
        }
    }

    /// Get the tiles out of this builder
    pub fn into_tiles(self) -> HexPointMap<T> {
        self.tiles
    }
}

pub trait Generate<In, Out>: Debug + Default {
    fn generate(
        &self,
        config: &WorldConfig,
        tiles: HexPointMap<In>,
    ) -> HexPointMap<Out>;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MagicGenerator;

impl Generate<ElevationMetadata, Tile> for MagicGenerator {
    fn generate(
        &self,
        _config: &WorldConfig,
        tiles: HexPointMap<ElevationMetadata>,
    ) -> HexPointMap<Tile> {
        tiles
            .into_iter()
            .map(|(pos, elev)| {
                Tile {
                    position: pos,
                    elevation: elev.elevation,
                    humidity: 0.0,
                    biome: Biome::Alpine,
                }
                .into_tuple()
            })
            .collect()
    }
}
