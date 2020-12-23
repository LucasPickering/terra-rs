mod biome;
mod elevation;
mod humidity;
mod lake;
mod ocean;
mod runoff;

use crate::{
    timed,
    util::NumRange,
    world::{
        generate::{
            biome::BiomePainter, elevation::ElevationGenerator,
            humidity::HumidityGenerator, lake::LakeGenerator,
            ocean::OceanGenerator, runoff::RunoffGenerator,
        },
        hex::{HexPoint, WorldMap},
        tile::{Tile, TileBuilder},
        WorldConfig,
    },
    NoiseFnConfig,
};
use log::info;
use noise::{MultiFractal, NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::fmt::{Debug, Display};

/// TODO
pub struct WorldBuilder {
    config: WorldConfig,
    rng: Pcg64,
    tiles: WorldMap<TileBuilder>,
}

impl WorldBuilder {
    pub fn new(config: WorldConfig) -> Self {
        // Initialize each tile
        let tiles = timed!("World initialization", {
            WorldMap::new(config.tile_radius, TileBuilder::new)
        });

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
    pub fn generate_world(mut self) -> WorldMap<Tile> {
        // Run each generation step. The order is very important!
        self.apply_generator(ElevationGenerator);
        self.apply_generator(HumidityGenerator);
        self.apply_generator(OceanGenerator);
        self.apply_generator(RunoffGenerator);
        self.apply_generator(LakeGenerator);
        self.apply_generator(BiomePainter);

        // Build each tile into its final value
        self.tiles.map(TileBuilder::build)
    }

    /// A helper to run a generation step on this builder.
    fn apply_generator(&mut self, generator: impl Generate) {
        timed!(
            &generator.to_string(),
            generator.generate(&self.config, &mut self.rng, &mut self.tiles)
        );
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
    fn generate(
        &self,
        config: &WorldConfig,
        rng: &mut impl Rng,
        tiles: &mut WorldMap<TileBuilder>,
    );
}

/// A wrapper around a noise function that makes it easy to use for generating
/// tile values. This is initialized for a particular function type, and
/// makes it easy to pass in a [HexPoint] and get out values in an arbitrary
/// output range.
#[derive(Clone, Debug)]
pub struct TileNoiseFn<F: NoiseFn<[f64; 3]>> {
    /// The noise generation function
    noise_fn: F,
    /// Exponent to apply to each noise value. This will be applied to values
    /// in the range [0,1], so exponents <1 bias upwards, and >1 bias
    /// downwards.
    exponent: f64,
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
        world_config: &WorldConfig,
        noise_fn: F,
        exponent: f64,
        output_range: NumRange<f64>,
    ) -> Self {
        let radius = world_config.tile_radius as f64;
        Self {
            noise_fn,
            exponent,
            // The noise functions expect input in [-1, 1], so we need this to
            // map our tile positions
            // tile_pos_range: NumRange::new(-radius, radius),
            tile_pos_range: NumRange::new(-100.0, 100.0),
            output_range,
        }
    }

    /// Helper to map one value in a [HexPoint] to [-1, 1].
    fn normalize_input(&self, value: i16) -> f64 {
        self.tile_pos_range
            .map(&Self::NOISE_FN_INPUT_RANGE, value as f64)
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
        rng: &mut impl Rng,
        fn_config: &NoiseFnConfig,
        output_range: NumRange<f64>,
    ) -> Self {
        // Configure the noise function
        let noise_fn = F::default()
            // Gen a new seed so that we get a different one per function
            .set_seed(rng.gen())
            .set_octaves(fn_config.octaves)
            .set_frequency(fn_config.frequency)
            .set_lacunarity(fn_config.lacunarity)
            .set_persistence(fn_config.persistence);

        Self::from_fn(world_config, noise_fn, fn_config.exponent, output_range)
    }
}

impl<F: NoiseFn<[f64; 3]>> NoiseFn<HexPoint> for TileNoiseFn<F> {
    fn get(&self, point: HexPoint) -> f64 {
        // Map each input value to [-1, 1]
        let normalized_input = [
            self.normalize_input(point.x()),
            self.normalize_input(point.y()),
            self.normalize_input(point.z()),
        ];
        let fn_output = self.noise_fn.get(normalized_input);
        Self::NOISE_FN_OUTPUT_RANGE
            .value(fn_output)
            // Map to [0,1] so we can apply the exponent
            .normalize()
            .apply(|val| val.powf(self.exponent))
            .map_to(self.output_range)
            .inner()
    }
}
