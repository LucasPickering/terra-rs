mod elevation;

use crate::{
    timed,
    world::{Biome, HasHexPosition, HexPoint, HexPointMap, Tile, WorldConfig},
};
pub use elevation::*;
use log::{debug, info};
use std::fmt::Debug;

pub struct WorldBuilder<T> {
    config: WorldConfig,
    tiles: HexPointMap<T>,
}

impl WorldBuilder<()> {
    pub fn new(config: WorldConfig) -> Self {
        // Initialize a set of tiles with no data
        let mut tiles = HexPointMap::new();
        let radius: isize = config.tile_radius as isize;

        // Some of these inserts will be duplicates, when `x == y`. It's easier
        // just to overwrite those instead of programming around them.
        for x in -radius..=radius {
            for y in -radius..=radius {
                let pos = HexPoint::new(x, y, -(x + y));
                tiles.insert(pos, ());
            }
        }

        // The final count should always be `4r^2 + 2r + 1`, where r is radius
        info!("Initialized world with {} tiles", tiles.len());
        Self { config, tiles }
    }
}

impl<T> WorldBuilder<T> {
    /// A helper to run a generation step on this builder, returning a new
    /// builder. Allows for chained `.generate` calls.
    pub fn apply_generator<U, G: Generate<T, U>>(
        self,
        generator: &G,
    ) -> WorldBuilder<U> {
        let (new_tiles, elapsed) =
            timed!(generator.generate(&self.config, self.tiles));
        debug!("{:?} took {:.3}s", generator, elapsed.as_secs_f32());

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

/// A type that generates some sort of data for the world. This takes in a set
/// of tiles that have some data generated, and generates new data for the
/// output. Generally there will be a series of generators chained together,
/// where each one adds some more data until the world is complete.
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
