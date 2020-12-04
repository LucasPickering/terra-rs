mod biome;
mod elevation;
mod humidity;

use crate::{
    timed,
    util::FloatRange,
    world::{
        config::NoiseFnConfig,
        generate::{
            biome::{BiomeMetadata, BiomePainter},
            elevation::ElevationGenerator,
            humidity::HumidityGenerator,
        },
        HasHexPosition, HexPoint, HexPointMap, Tile, WorldConfig,
    },
};
use log::{debug, info};
use noise::{MultiFractal, NoiseFn, Seedable};
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

/// A wrapper around a noise function that makes it easy to use for generating
/// tile values. This is initialized for a particular function type, and
/// makes it easy to pass in a [HexPoint] and get out values in an arbitrary
/// output range.
#[derive(Clone, Debug)]
pub struct TileNoiseFn<F: NoiseFn<[f64; 3]>> {
    /// The noise generation function
    noise_fn: F,
    /// The range of tile position values. Used to map the input.
    tile_pos_range: FloatRange,
    output_range: FloatRange,
}

impl<F: NoiseFn<[f64; 3]>> TileNoiseFn<F> {
    /// The output range of the internal noise function. Used to map our
    /// input values to the noise function's input values.
    const NOISE_FN_INPUT_RANGE: FloatRange = FloatRange::new(-1.0, 1.0);
    /// The output range of the internal noise function. Used to map the noise
    /// values to our own output range.
    const NOISE_FN_OUTPUT_RANGE: FloatRange = FloatRange::new(-1.0, 1.0);

    /// Initialize a wrapper around the given function.
    fn from_fn(
        world_radius: usize,
        noise_fn: F,
        output_range: FloatRange,
    ) -> Self {
        let radius_f = world_radius as f64;
        Self {
            noise_fn,
            tile_pos_range: FloatRange::new(-radius_f, radius_f),
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
        output_range: FloatRange,
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
