mod biome;
mod elevation;
mod humidity;
mod ocean;

use crate::{
    config::NoiseFnConfig,
    timed,
    util::NumRange,
    world::{
        generate::{
            biome::BiomePainter, elevation::ElevationGenerator,
            humidity::HumidityGenerator, ocean::OceanGenerator,
        },
        hex::{HexPoint, HexPointMap},
        tile::{TileBuilder, TileMap},
        WorldConfig,
    },
};
use log::{debug, info};
use noise::{MultiFractal, NoiseFn, Seedable};
use rand::SeedableRng;
use rand_pcg::Pcg64;
use std::fmt::{Debug, Display};

pub struct WorldBuilder {
    config: WorldConfig,
    rng: Pcg64,
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
        Self {
            config,
            rng: Pcg64::seed_from_u64(config.seed),
            tiles,
        }
    }

    /// Generate a world by running a series of generation steps sequentially.
    /// Must be run from a blank slate. Outputs the finalized set of tiles.
    pub fn generate_world(mut self) -> TileMap {
        let config = self.config;

        // Run each generation step. The order is very important!
        self.apply_generator(ElevationGenerator::new(&config));
        self.apply_generator(HumidityGenerator::new(&config));
        self.apply_generator(BiomePainter);
        self.apply_generator(OceanGenerator);

        // Build each tile into its final value
        self.tiles
            .into_iter()
            .map(|(pos, tile)| (pos, tile.build()))
            .collect()
    }

    /// A helper to run a generation step on this builder.
    fn apply_generator(&mut self, generator: impl Generate) {
        let ((), elapsed) =
            timed!(generator.generate(&mut self.tiles, &mut self.rng));
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
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>, rng: &mut Pcg64);
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
    fn from_fn(noise_fn: F, output_range: NumRange<f64>) -> Self {
        Self {
            noise_fn,
            // The noise function doesn't give interesting output for whole
            // number inputs, so we need to map this down to decimal numbers
            tile_pos_range: NumRange::new(-100.0, 100.0),
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
            // Mask off the top 32 bits
            .set_seed((world_config.seed & 0xffffffff) as u32)
            .set_octaves(fn_config.octaves)
            .set_frequency(fn_config.frequency)
            .set_lacunarity(fn_config.lacunarity)
            .set_persistence(fn_config.persistence);

        Self::from_fn(noise_fn, output_range)
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
