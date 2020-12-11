mod biome;
mod elevation;
mod humidity;

use crate::{
    config::NoiseFnConfig,
    timed,
    util::NumRange,
    world::{
        generate::{
            biome::BiomePainter, elevation::ElevationGenerator,
            humidity::HumidityGenerator,
        },
        Biome, HasHexPosition, HexPoint, HexPointMap, Tile, WorldConfig,
    },
};
use log::{debug, info};
use noise::{MultiFractal, NoiseFn, Seedable};
use std::fmt::Display;

pub struct WorldBuilder {
    config: WorldConfig,
    tiles: HexPointMap<TileBuilder>,
}

impl WorldBuilder {
    pub fn new(config: WorldConfig) -> Self {
        // Initialize a set of tiles with no data
        let mut tiles = HexPointMap::new();
        let radius: isize = config.tile_radius as isize;

        // Initialize each tile
        for x in -radius..=radius {
            for y in -radius..=radius {
                // x+y+z == 0 always, so we can derive z from x & y.
                let pos = HexPoint::new(x, y);

                // There will be duplicate positions, when `x == y`. Avoid an
                // insert for those.
                tiles.entry(pos).or_insert_with(|| TileBuilder::new(pos));
            }
        }

        // The final count should always be `4r^2 + 2r + 1`, where r is radius
        info!("Initialized world with {} tiles", tiles.len());
        Self { config, tiles }
    }

    /// Generate a world by running a series of generation steps sequentially.
    /// Must be run from a blank slate. Outputs the finalized set of tiles.
    pub fn generate_world(mut self) -> HexPointMap<Tile> {
        let config = self.config;

        // Run each generation step. The order is very important!
        self.apply_generator(ElevationGenerator::new(&config));
        self.apply_generator(HumidityGenerator::new(&config));
        self.apply_generator(BiomePainter);

        // Build each tile into its final value
        self.tiles
            .into_iter()
            .map(|(pos, tile)| (pos, tile.build()))
            .collect()
    }

    /// A helper to run a generation step on this builder.
    fn apply_generator(&mut self, generator: impl Generate) {
        let ((), elapsed) = timed!(generator.generate(&mut self.tiles));
        debug!("{} took {}ms", generator, elapsed.as_millis());
    }
}

/// A type that generates some sort of data for the world. This takes in a set
/// of tiles that have some data generated, and mutates the tiles to add new
/// data. Generally there will be a series of generators chained together,
/// where each one adds some more data until the world is complete.
trait Generate: Display {
    /// Add new data to the existing tiles. The given map should never be
    /// inserted into or removed from, and the keys should never be changed.
    /// Only the values (tiles) should be mutated!
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>);
}

/// A wrapper around a noise function that makes it easy to use for generating
/// tile values. This is initialized for a particular function type, and
/// makes it easy to pass in a [HexPoint] and get out values in an arbitrary
/// output range.
#[derive(Clone, Debug)]
pub struct TileNoiseFn<F: NoiseFn<[f64; 3]>> {
    /// The noise generation function
    noise_fn: F,
    /// The range of tile position values. Used to map the input.
    tile_pos_range: NumRange<f64>,
    output_range: NumRange<f64>,
}

impl<F: NoiseFn<[f64; 3]>> TileNoiseFn<F> {
    /// The output range of the internal noise function. Used to map our
    /// input values to the noise function's input values.
    const NOISE_FN_INPUT_RANGE: NumRange<f64> = NumRange::new(-1.0, 1.0);
    /// The output range of the internal noise function. Used to map the noise
    /// values to our own output range.
    const NOISE_FN_OUTPUT_RANGE: NumRange<f64> = NumRange::new(-1.0, 1.0);

    /// Initialize a wrapper around the given function.
    fn from_fn(
        world_radius: usize,
        noise_fn: F,
        output_range: NumRange<f64>,
    ) -> Self {
        let radius_f = world_radius as f64;
        Self {
            noise_fn,
            tile_pos_range: NumRange::new(-radius_f, radius_f),
            output_range,
        }
    }

    /// Helper to map one value in a [HexPoint] to [-1, 1].
    fn normalize_input(&self, value: isize) -> f64 {
        self.tile_pos_range
            .map_to(&Self::NOISE_FN_INPUT_RANGE, value as f64)
    }
}

impl<F: Default + Seedable + MultiFractal + NoiseFn<[f64; 3]>> TileNoiseFn<F> {
    /// Initialize a new function for some underlying noise fn type.
    ///
    /// ### Arguments
    /// - `world_config` - The overall world config, needed for seed and world
    /// radius.
    /// - `fn_config` - Configuration for the underlying noise fn.
    /// - `output_range` - The output range of this function. Noise values will
    /// be mapped to this range during generation.
    pub fn new(
        world_config: &WorldConfig,
        fn_config: &NoiseFnConfig,
        output_range: NumRange<f64>,
    ) -> Self {
        // Configure the noise function
        let noise_fn = F::default()
            .set_seed(world_config.seed)
            .set_octaves(fn_config.octaves)
            .set_frequency(fn_config.frequency)
            .set_lacunarity(fn_config.lacunarity)
            .set_persistence(fn_config.persistence);

        Self::from_fn(world_config.tile_radius, noise_fn, output_range)
    }
}

impl<F: NoiseFn<[f64; 3]>> NoiseFn<HexPoint> for TileNoiseFn<F> {
    fn get(&self, point: HexPoint) -> f64 {
        // Map each input value to [-1, 1]
        let normalized_input = [
            self.normalize_input(point.x),
            self.normalize_input(point.y),
            self.normalize_input(point.z),
        ];
        let normalized_output = self.noise_fn.get(normalized_input);
        Self::NOISE_FN_OUTPUT_RANGE
            .map_to(&self.output_range, normalized_output)
    }
}

/// A partially built [Tile]. This should only be used while the world is being
/// generated. After generation is complete, only [Tile] should be used. All the
/// fields on this type, other than `position`, have a getter and a setter.
/// Since the fields may not be defined, the getters all panic if the field
/// has not be set. This makes it easy to catch bugs where we're trying to use
/// world values that haven't been generated yet.
#[derive(Copy, Clone, Debug)]
struct TileBuilder {
    position: HexPoint,
    elevation: Option<f64>,
    humidity: Option<f64>,
    biome: Option<Biome>,
}

impl TileBuilder {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            elevation: None,
            humidity: None,
            biome: None,
        }
    }

    pub fn build(self) -> Tile {
        Tile {
            position: self.position(),
            elevation: self.elevation(),
            humidity: self.humidity(),
            biome: self.biome(),
        }
    }

    /// Get this tile's elevation. Panics if elevation has not been set yet.
    pub fn elevation(&self) -> f64 {
        self.elevation.unwrap()
    }

    /// Set the elevation for this tile.
    pub fn set_elevation(&mut self, elevation: f64) {
        self.elevation = Some(elevation);
    }

    /// Get this tile's humidity. Panics if humidity has not been set yet.
    pub fn humidity(&self) -> f64 {
        self.humidity.unwrap()
    }

    /// Set the humidity for this tile.
    pub fn set_humidity(&mut self, humidity: f64) {
        self.humidity = Some(humidity);
    }

    /// Get this tile's biome. Panics if biome has not been set yet.
    pub fn biome(&self) -> Biome {
        self.biome.unwrap()
    }

    /// Set the biome for this tile.
    pub fn set_biome(&mut self, biome: Biome) {
        self.biome = Some(biome);
    }
}

impl HasHexPosition for TileBuilder {
    fn position(&self) -> HexPoint {
        self.position
    }
}
