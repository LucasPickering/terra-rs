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
        hex::{
            HasHexPosition, HexAxialDirection, HexDirection, HexPoint,
            HexPointMap,
        },
        Biome, BiomeType, GeoFeature, Meter, Tile, World, WorldConfig,
    },
};
use anyhow::{anyhow, Context};
use fnv::FnvBuildHasher;
use log::info;
use noise::{MultiFractal, NoiseFn, Seedable};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::{
    cmp,
    collections::{HashMap, HashSet},
    fmt::Debug,
};

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
        // TODO config validation

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

    /// Get the prevailing wind direction of this world. Returns an error if
    /// the wind direction hasn't be initialized yet.
    pub fn wind_direction(&self) -> anyhow::Result<HexAxialDirection> {
        self.wind_direction
            .ok_or_else(|| anyhow!("wind direction not initialized"))
    }

    /// Generate a world by running a series of generation steps sequentially.
    /// Must be run from a blank slate. Outputs the finalized set of tiles.
    pub fn generate_world(mut self) -> anyhow::Result<HexPointMap<Tile>> {
        // Run each generation step. The order is very important!
        self.apply_generator(ElevationGenerator)?;
        self.apply_generator(WindGenerator)?;
        self.apply_generator(OceanGenerator)?;
        self.apply_generator(RainfallGenerator)?;
        self.apply_generator(RunoffGenerator)?;
        self.apply_generator(LakeGenerator)?;
        self.apply_generator(BiomeGenerator)?;

        // Build each tile into its final value
        self.tiles
            .into_iter()
            .map(|(pos, tile)| Ok((pos, tile.build()?)))
            .collect()
    }

    /// A helper to run a generation step on this builder.
    fn apply_generator(
        &mut self,
        generator: impl Debug + Generate,
    ) -> anyhow::Result<()> {
        timed!(&format!("{:?}", generator), generator.generate(self))
            .with_context(|| format!("error in {:?}", generator))
    }
}

/// A type that generates some sort of data for the world. This takes in a set
/// of tiles that have some data generated, and mutates the tiles to add new
/// data. Generally there will be a series of generators chained together,
/// where each one adds some more data until the world is complete.
trait Generate {
    /// Apply some generation step to the given world. This can mutate the
    /// world's tiles, but can never add/remove tiles, or change their positions
    /// in any way. Any failure that occurs in this function should be
    /// considered an internal failure, meaning a bug in the code, rather than
    /// anything invalid about the input. Any implementation of this function
    /// _should_ be able to handle any input without returning an error, so
    /// any error is considered exceptional.
    fn generate(&self, world: &mut WorldBuilder) -> anyhow::Result<()>;
}

/// A partially built [Tile]. This should only be used while the world is being
/// generated. After generation is complete, only [Tile] should be used. All the
/// fields on this type, other than `position`, have a getter and a setter.
/// Since the fields may not be defined, the getters all return results that
/// error if the field hasn't been set. This makes it easy to catch bugs where
/// we're trying to use world values that haven't been generated yet.
#[derive(Clone, Debug)]
pub struct TileBuilder {
    position: HexPoint,
    elevation: Option<Meter>,
    rainfall: Option<Meter3>,
    biome: Option<Biome>,
    runoff: Option<Meter3>,
    runoff_egress: Option<HashMap<HexDirection, Meter3, FnvBuildHasher>>,
    features: HashSet<GeoFeature, FnvBuildHasher>,
}

impl TileBuilder {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            elevation: None,
            rainfall: None,
            runoff: None,
            runoff_egress: None,
            biome: None,
            features: HashSet::default(),
        }
    }

    /// Finalize this builder to create a [Tile]. Returns an error if any fields
    /// on this builder are uninitialized.
    pub fn build(self) -> anyhow::Result<Tile> {
        let position = self.position;
        Ok(Tile {
            position,
            elevation: self.elevation()?,
            rainfall: self.rainfall()?,
            runoff: self.runoff()?,
            biome: self.biome()?,
            features: self.features,
            runoff_egress: self.runoff_egress.ok_or_else(|| {
                anyhow!("runoff_egress not initialized for {}", position)
            })?,
        })
    }

    /// See [Tile::elevation]. Returns an error if elevation is unset.
    pub fn elevation(&self) -> anyhow::Result<Meter> {
        self.elevation.ok_or_else(|| {
            anyhow!("elevation not initialized for {}", self.position)
        })
    }

    /// Set the elevation for this tile. Returns an error if the elevation value
    /// is out of [World::ELEVATION_RANGE].
    pub fn set_elevation(&mut self, elevation: Meter) -> anyhow::Result<()> {
        World::ELEVATION_RANGE.ensure_contains(elevation)?;
        self.elevation = Some(elevation);
        Ok(())
    }

    /// See [Tile::rainfall]. Returns an error if rainfall is unset.
    pub fn rainfall(&self) -> anyhow::Result<Meter3> {
        self.rainfall.ok_or_else(|| {
            anyhow!("rainfall not initialized for {}", self.position)
        })
    }

    /// Set the rainfall for this tile. Returns an error if the given value is
    /// negative.
    pub fn set_rainfall(&mut self, rainfall: Meter3) -> anyhow::Result<()> {
        if rainfall >= Meter3(0.0) {
            self.rainfall = Some(rainfall);
            Ok(())
        } else {
            Err(anyhow!("cannot set negative rainfall {}"))
        }
    }

    /// See [Tile::humidity]. Returns an error if humidity is unset.
    pub fn humidity(&self) -> anyhow::Result<f64> {
        let rainfall = self
            .rainfall()
            .with_context(|| "failed to calculate humidity")?;
        Ok(World::RAINFALL_SOFT_RANGE
            .value(rainfall)
            .clamp()
            .convert::<f64>()
            .normalize()
            .inner())
    }

    /// Set the biome for this tile.
    pub fn set_biome(&mut self, biome: Biome) {
        self.biome = Some(biome);
    }

    /// See [Tile::runoff]. Returns an error if runoff is unset.
    pub fn runoff(&self) -> anyhow::Result<Meter3> {
        self.runoff.ok_or_else(|| {
            anyhow!("runoff not initialized for {}", self.position)
        })
    }

    /// Add some amount of runoff to this tile. Returns an error if the amount
    /// is negative or runoff is uninitialized.
    pub fn add_runoff(&mut self, runoff: Meter3) -> anyhow::Result<()> {
        if runoff >= Meter3(0.0) {
            self.runoff = Some(self.runoff()? + runoff);
            Ok(())
        } else {
            Err(anyhow!("cannot add negative runoff {}", runoff))
        }
    }

    /// Set the runoff level of this tile (any existing runoff will be deleted).
    /// Returns an error if the runoff value is negative.
    pub fn set_runoff(&mut self, runoff: Meter3) -> anyhow::Result<()> {
        if runoff >= Meter3(0.0) {
            self.runoff = Some(runoff);
            Ok(())
        } else {
            Err(anyhow!("cannot set negative runoff {}", runoff))
        }
    }

    /// Reset the runoff on this tile to 0 and return whatever amount was here.
    /// Returns an error if runoff is unset.
    pub fn clear_runoff(&mut self) -> anyhow::Result<Meter3> {
        let runoff = self.runoff()?;
        self.runoff = Some(Meter3(0.0));
        Ok(runoff)
    }

    /// Set the runoff_egress map, which tells us how much runoff this tile
    /// pushed out to each of its neighbors.
    pub fn set_runoff_egress(
        &mut self,
        runoff_egress: HashMap<HexDirection, Meter3, FnvBuildHasher>,
    ) {
        self.runoff_egress = Some(runoff_egress);
    }

    /// See [Tile::biome]. Returns an error if biome is unset.
    pub fn biome(&self) -> anyhow::Result<Biome> {
        self.biome.ok_or_else(|| {
            anyhow!("biome not initialized for {}", self.position)
        })
    }

    /// Get this tile's biome as an `Option`, meaning if the biome is unset, we
    /// return `None` (as opposed to [Self::biome], which returns an error).
    pub fn biome_opt(&self) -> Option<Biome> {
        self.biome
    }

    /// Convenience method to check if this tile is water. Will return false if
    /// the tile is land OR has no biome set.
    pub fn is_water_biome(&self) -> bool {
        match self.biome {
            Some(biome) => biome.biome_type() == BiomeType::Water,
            None => false,
        }
    }

    /// Add a new geographic feature to this tile. Returns an error if the tile
    /// already has that feature.
    pub fn add_feature(&mut self, feature: GeoFeature) -> anyhow::Result<()> {
        let is_new = self.features.insert(feature);
        if is_new {
            Ok(())
        } else {
            Err(anyhow!(
                "feature {:?} already exists for {:?}",
                feature,
                self
            ))
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
