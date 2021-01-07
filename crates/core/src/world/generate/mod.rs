mod biome;
mod elevation;
mod lake;
mod ocean;
mod rainfall;
mod runoff;
mod wind;

use crate::{
    config::NoiseFnConfig,
    timed,
    util::{self, Meter3, NumRange, Rangeable},
    world::{
        generate::{
            biome::BiomeGenerator, elevation::ElevationGenerator,
            lake::LakeGenerator, ocean::OceanGenerator,
            rainfall::RainfallGenerator, runoff::RunoffGenerator,
            wind::WindGenerator,
        },
        hex::{HasHexPosition, HexAxialDirection, HexPoint, HexPointMap},
        Biome, BiomeType, Meter, Tile, World, WorldConfig,
    },
};
use fnv::FnvBuildHasher;
use log::info;
use noise::{MultiFractal, NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::{cmp, fmt::Debug};

/// A container for generating a new world. This applies a series of generators
/// in sequence to create the world. These fields are public to allow for
/// disjoint borrowing of multiple fields at once.
///
/// In the generator code that this builder calls, you'll see a lot of
/// algorithms that tend to generate new values in a separate map, then do a 2nd
/// pass to put those values into this map. This is necessary because a lot of
/// them need to reference multiple tiles at once, and once you have a mutable
/// reference to one tile in a collection, you can't grab references to any
/// other items. But it turns out that doing it this way is usually a lot faster
/// and simpler than using a hack like `Rc<RefCell<_>>`.
pub struct WorldBuilder {
    /// This config determinisitically controls world config, meaning two
    /// worlds with the same config will always be identical (provided they
    /// were generated on the same version of the code).
    ///
    /// This is public to allow for disjoint borrowing, but please **do not
    /// mutate the config**.
    pub config: WorldConfig,

    /// RNG provider.
    pub rng: Pcg64,

    /// All the tiles in the world. These individual tiles will be mutated
    /// during world generation, but tiles can never be added/removed/moved!
    pub tiles: HexPointMap<TileBuilder>,

    /// Direction of the world's prevailing wind. Initialized by
    /// [WindGenerator], and is guaranteed to be populated after that.
    pub wind_direction: Option<HexAxialDirection>,
}

impl WorldBuilder {
    pub fn new(config: WorldConfig) -> Self {
        // Initialize each tile
        let tiles = timed!("World initialization", {
            let capacity = util::world_len(config.radius);
            let mut map = HexPointMap::with_capacity_and_hasher(
                capacity,
                FnvBuildHasher::default(),
            );

            // Initialize a set of tiles with no data
            let r = config.radius as i16;
            for x in -r..=r {
                // If we just do [-r,r] for y as well, then we end up with a
                // diamond pattern instead of a super hexagon
                // https://www.redblobgames.com/grids/hexagons/#range
                let y_min = cmp::max(-r, -x - r);
                let y_max = cmp::min(r, -x + r);
                for y in y_min..=y_max {
                    let pos = HexPoint::new(x, y);
                    map.insert(pos, TileBuilder::new(pos));
                }
            }

            debug_assert_eq!(map.len(), capacity, "expected 3r²+3r+1 tiles");
            map
        });

        // The final count should always be `4r^2 + 2r + 1`, where r is radius
        info!("Initialized world with {} tiles", tiles.len());
        Self {
            config,
            rng: Pcg64::seed_from_u64(config.seed),
            tiles,
            wind_direction: None,
        }
    }

    /// Generate a world by running a series of generation steps sequentially.
    /// Must be run from a blank slate. Outputs the finalized set of tiles.
    pub fn generate_world(mut self) -> HexPointMap<Tile> {
        // Run each generation step. The order is very important!
        self.apply_generator(ElevationGenerator);
        self.apply_generator(WindGenerator);
        self.apply_generator(OceanGenerator);
        self.apply_generator(RainfallGenerator);
        self.apply_generator(RunoffGenerator);
        self.apply_generator(LakeGenerator);
        self.apply_generator(BiomeGenerator);

        // Build each tile into its final value
        self.tiles
            .into_iter()
            .map(|(pos, tile)| (pos, tile.build()))
            .collect()
    }

    /// A helper to run a generation step on this builder.
    fn apply_generator(&mut self, generator: impl Debug + Generate) {
        timed!(&format!("{:?}", generator), generator.generate(self));
    }
}

/// A type that generates some sort of data for the world. This takes in a set
/// of tiles that have some data generated, and mutates the tiles to add new
/// data. Generally there will be a series of generators chained together,
/// where each one adds some more data until the world is complete.
trait Generate {
    /// Apply some generation step to the given world. This can mutate the
    /// world's tiles, but can never add/remove tiles, or change their positions
    /// in any way.
    fn generate(&self, world: &mut WorldBuilder);
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
    elevation: Option<Meter>,
    rainfall: Option<Meter3>,
    biome: Option<Biome>,
    runoff: Option<Meter3>,
}

impl TileBuilder {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            elevation: None,
            rainfall: None,
            biome: None,
            runoff: None,
        }
    }

    pub fn build(self) -> Tile {
        Tile {
            position: self.position,
            elevation: self.elevation.unwrap(),
            rainfall: self.rainfall.unwrap(),
            biome: self.biome.unwrap(),
            // This will still be uninitialized for ocean tiles
            runoff: self.runoff.unwrap_or(Meter3(0.0)),
        }
    }

    /// See [Tile::elevation]
    pub fn elevation(&self) -> Option<Meter> {
        self.elevation
    }

    /// Set the elevation for this tile.
    pub fn set_elevation(&mut self, elevation: Meter) {
        self.elevation = Some(elevation);
    }

    /// See [Tile::rainfall]
    pub fn rainfall(&self) -> Option<Meter3> {
        self.rainfall
    }

    /// Set the rainfall for this tile.
    pub fn set_rainfall(&mut self, rainfall: Meter3) {
        self.rainfall = Some(rainfall);
    }

    /// See [Tile::humidity]
    pub fn humidity(&self) -> Option<f64> {
        self.rainfall.map(|rainfall| {
            World::RAINFALL_SOFT_RANGE
                .value(rainfall)
                .clamp()
                .convert::<f64>()
                .normalize()
                .inner()
        })
    }

    /// See [Tile::biome]
    pub fn biome(&self) -> Option<Biome> {
        self.biome
    }

    /// Set the biome for this tile.
    pub fn set_biome(&mut self, biome: Biome) {
        self.biome = Some(biome);
    }

    /// See [Tile::runoff]
    pub fn runoff(&self) -> Option<Meter3> {
        self.runoff
    }

    /// Add some amount of runoff to this tile. Amount must be non-negative!
    /// Also panics if runoff is uninitialized.
    pub fn add_runoff(&mut self, runoff: Meter3) {
        assert!(runoff >= Meter3(0.0), "Must add non-negative runoff");
        self.runoff = Some(self.runoff.unwrap() + runoff);
    }

    /// Set the runoff level of this tile (any existing runoff will be deleted).
    pub fn set_runoff(&mut self, runoff: Meter3) {
        assert!(
            runoff >= Meter3(0.0),
            "Must set runoff to non-negative value"
        );
        self.runoff = Some(runoff);
    }

    /// Reset the runoff on this tile to 0 and return whatever amount was here.
    /// Panics if runoff is uninitialized.
    pub fn clear_runoff(&mut self) -> Meter3 {
        let runoff = self
            .runoff
            .expect("Runoff has not been initialized, cannot clear it");
        self.runoff = Some(Meter3(0.0));
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
///
/// This type can optionally also do transparent conversions on the output type,
/// e.g. if you are using a newtype that wraps `f64`.
#[derive(Clone, Debug)]
pub struct TileNoiseFn<F: NoiseFn<[f64; 3]>, T: Rangeable<f64> = f64> {
    /// The noise generation function
    noise_fn: F,
    /// Exponent to apply to each noise value. This will be applied to values
    /// in the range [0,1], so exponents <1 bias upwards, and >1 bias
    /// downwards.
    exponent: f64,
    output_range: NumRange<T, f64>,
}

impl<F: NoiseFn<[f64; 3]>, T: Rangeable<f64>> TileNoiseFn<F, T> {
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
        output_range: NumRange<T, f64>,
    ) -> Self {
        Self {
            noise_fn,
            exponent,
            output_range,
        }
    }

    /// Get the function output at the given point
    pub fn get(&self, point: HexPoint) -> T {
        // See INPUT_SCALE doc comment for why we need it
        let scaled_input = [
            point.x() as f64 / Self::INPUT_SCALE,
            point.y() as f64 / Self::INPUT_SCALE,
            point.z() as f64 / Self::INPUT_SCALE,
        ];
        let fn_output = self.noise_fn.get(scaled_input);
        Self::NOISE_FN_OUTPUT_RANGE
            .value(fn_output)
            // Map to [0,1] so we can apply the exponent
            .normalize()
            .apply(|val| val.powf(self.exponent))
            .convert() // f64 -> T
            .map_to(self.output_range)
            .inner()
    }
}

impl<
        F: Default + Seedable + MultiFractal + NoiseFn<[f64; 3]>,
        T: Rangeable<f64>,
    > TileNoiseFn<F, T>
{
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
        output_range: NumRange<T, f64>,
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
