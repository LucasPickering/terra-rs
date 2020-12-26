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
        hex::{HasHexPosition, HexPoint, WorldMap},
        Biome, BiomeType, Tile, WorldConfig,
    },
    NoiseFnConfig,
};
use log::info;
use noise::{MultiFractal, NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::fmt::Debug;

/// A container for generating a new world. This applies a series of generators
/// in sequence to create the world.
///
/// In the generator code that this builder calls, you'll see a lot of
/// algorithms that tend to generate new values in a separate map, then do a 2nd
/// pass to put those values into this map. This is necessary because a lot of
/// them need to reference multiple tiles at once, and once you have a mutable
/// reference to one tile in a collection, you can't grab references to any
/// other items. But it turns out that doing it this way is usually a lot faster
/// and simpler than using a hack like `Rc<RefCell<_>>`.
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
    fn apply_generator(&mut self, generator: impl Debug + Generate) {
        timed!(
            &format!("{:?}", generator),
            generator.generate(&self.config, &mut self.rng, &mut self.tiles)
        );
    }
}

/// A type that generates some sort of data for the world. This takes in a set
/// of tiles that have some data generated, and mutates the tiles to add new
/// data. Generally there will be a series of generators chained together,
/// where each one adds some more data until the world is complete.
trait Generate {
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

/// A partially built [Tile]. This should only be used while the world is being
/// generated. After generation is complete, only [Tile] should be used. All the
/// fields on this type, other than `position`, have a getter and a setter.
/// Since the fields may not be defined, the getters all panic if the field
/// has not be set. This makes it easy to catch bugs where we're trying to use
/// world values that haven't been generated yet.
#[derive(Clone, Debug)] // intentionally omit Copy because it may not be possible in the future
pub struct TileBuilder {
    position: HexPoint,
    elevation: Option<f64>,
    humidity: Option<f64>,
    biome: Option<Biome>,
    runoff: f64,
}

impl TileBuilder {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            elevation: None,
            humidity: None,
            biome: None,
            runoff: 0.0,
        }
    }

    pub fn build(self) -> Tile {
        Tile {
            position: self.position,
            elevation: self.elevation.unwrap(),
            humidity: self.humidity.unwrap(),
            biome: self.biome.unwrap(),
            runoff: self.runoff,
        }
    }

    /// Get this tile's elevation. Panics if elevation has not been set yet.
    pub fn elevation(&self) -> Option<f64> {
        self.elevation
    }

    /// Set the elevation for this tile.
    pub fn set_elevation(&mut self, elevation: f64) {
        self.elevation = Some(elevation);
    }

    /// Get this tile's humidity. Panics if humidity has not been set yet.
    pub fn humidity(&self) -> Option<f64> {
        self.humidity
    }

    /// Set the humidity for this tile.
    pub fn set_humidity(&mut self, humidity: f64) {
        self.humidity = Some(humidity);
    }

    /// Get this tile's biome. Panics if biome has not been set yet.
    pub fn biome(&self) -> Option<Biome> {
        self.biome
    }

    /// Set the biome for this tile.
    pub fn set_biome(&mut self, biome: Biome) {
        self.biome = Some(biome);
    }

    /// Amount of runoff CURRENTLY on this tile (NOT the total amount that has
    /// crossed over this tile).
    pub fn runoff(&self) -> f64 {
        self.runoff
    }

    /// Add some amount of runoff to this tile. Amount must be non-negative!
    pub fn add_runoff(&mut self, runoff: f64) {
        assert!(runoff >= 0.0, "Must add non-negative runoff");
        self.runoff += runoff;
    }

    pub fn set_runoff(&mut self, runoff: f64) {
        assert!(runoff >= 0.0, "Must set runoff to non-negative value");
        self.runoff = runoff;
    }

    /// Reset the runoff on this tile to 0 and return whatever amount was here
    pub fn clear_runoff(&mut self) -> f64 {
        let runoff = self.runoff;
        self.runoff = 0.0;
        runoff
    }

    /// Convenience method to check if this tile is water. Will return false if
    /// the tile is land OR has no biome set.
    pub fn is_water(&self) -> bool {
        match self.biome {
            Some(biome) => biome.biome_type() == BiomeType::Water,
            None => false,
        }
    }
}

impl HasHexPosition for TileBuilder {
    fn position(&self) -> HexPoint {
        self.position
    }
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
    output_range: NumRange<f64>,
}

impl<F: NoiseFn<[f64; 3]>> TileNoiseFn<F> {
    /// If we used the full values from the input, our frequencies would have
    /// to be stupid low to get resonable looking output, so we scale them
    /// down by this factor
    const INPUT_SCALE: f64 = 100.0;
    /// The output range of the internal noise function. Used to map the noise
    /// values to our own output range.
    const NOISE_FN_OUTPUT_RANGE: NumRange<f64> = NumRange::new(-1.0, 1.0);

    /// Initialize a wrapper around the given function.
    fn from_fn(
        noise_fn: F,
        exponent: f64,
        output_range: NumRange<f64>,
    ) -> Self {
        Self {
            noise_fn,
            exponent,
            output_range,
        }
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

        Self::from_fn(noise_fn, fn_config.exponent, output_range)
    }
}

impl<F: NoiseFn<[f64; 3]>> NoiseFn<HexPoint> for TileNoiseFn<F> {
    fn get(&self, point: HexPoint) -> f64 {
        // See INPUT_SCALE doc comment for why we need it
        let normalized_input = [
            point.x() as f64 / Self::INPUT_SCALE,
            point.y() as f64 / Self::INPUT_SCALE,
            point.z() as f64 / Self::INPUT_SCALE,
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
