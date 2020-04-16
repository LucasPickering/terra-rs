mod biome;
mod elevation;
mod humidity;

use crate::{
    timed,
    world::{
        generate::{
            biome::{BiomeMetadata, BiomePainter},
            elevation::ElevationGenerator,
            humidity::HumidityGenerator,
        },
        HasHexPosition, HexPoint, HexPointMap, Tile, WorldConfig,
    },
};
use log::{debug, info};
use std::fmt::{self, Display, Formatter};

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

    /// Generate a world by running a series of generation steps sequentially.
    /// Must be run from a blank slate. Outputs the finalized set of tiles.
    pub fn generate_world(self) -> HexPointMap<Tile> {
        let config = self.config;
        self.apply_generator(&ElevationGenerator::new(&config))
            .apply_generator(&HumidityGenerator::new(&config))
            .apply_generator(&BiomePainter)
            .apply_generator(&MagicGenerator)
            .tiles
    }
}

impl<T> WorldBuilder<T> {
    /// A helper to run a generation step on this builder, returning a new
    /// builder. Allows for chained calls on an instance.
    fn apply_generator<U, G: Generate<T, U>>(
        self,
        generator: &G,
    ) -> WorldBuilder<U> {
        let (new_tiles, elapsed) = timed!(generator.generate(self.tiles));
        debug!("{} took {}ms", generator, elapsed.as_millis());

        WorldBuilder {
            config: self.config,
            tiles: new_tiles,
        }
    }
}

/// A type that generates some sort of data for the world. This takes in a set
/// of tiles that have some data generated, and generates new data for the
/// output. Generally there will be a series of generators chained together,
/// where each one adds some more data until the world is complete.
pub trait Generate<In, Out>: Display {
    fn generate(&self, tiles: HexPointMap<In>) -> HexPointMap<Out>;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MagicGenerator;

impl Display for MagicGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MagicGenerator")?;
        Ok(())
    }
}

impl Generate<BiomeMetadata, Tile> for MagicGenerator {
    fn generate(&self, tiles: HexPointMap<BiomeMetadata>) -> HexPointMap<Tile> {
        tiles
            .into_iter()
            .map(|(pos, data)| {
                Tile {
                    position: pos,
                    elevation: data.elevation,
                    humidity: data.humidity,
                    biome: data.biome,
                }
                .into_tuple()
            })
            .collect()
    }
}
