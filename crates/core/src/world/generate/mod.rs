mod biome;
mod elevation;
mod noise;
mod ocean;
mod rainfall;
mod runoff;
mod water_feature;
mod wind;

use crate::{
    timed, unwrap,
    util::{self, Meter3},
    world::{
        generate::{
            biome::BiomeGenerator,
            elevation::ElevationGenerator,
            ocean::OceanGenerator,
            rainfall::RainfallGenerator,
            runoff::{RunoffGenerator, RunoffPattern},
            water_feature::WaterFeatureGenerator,
            wind::WindGenerator,
        },
        hex::{
            HasHexPosition, HexAxialDirection, HexDirection, HexDirectionMap,
            HexPoint, HexPointMap,
        },
        Biome, BiomeType, GeoFeature, Meter, Tile, World, WorldConfig,
    },
};
use fnv::FnvBuildHasher;
use log::info;
use rand::SeedableRng;
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
    /// Initialize a builder that will construct a new world. **This assumes
    /// that the given config is already validated!**
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

    /// Get the prevailing wind direction of this world. Panics if
    /// the wind direction hasn't be initialized yet.
    pub fn wind_direction(&self) -> HexAxialDirection {
        self.wind_direction.expect("wind direction not initialized")
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
        self.apply_generator(WaterFeatureGenerator);
        self.apply_generator(BiomeGenerator);

        // Build each tile into its final value
        self.tiles
            .into_iter()
            .map(|(pos, tile)| (pos, tile.build()))
            .collect()
    }

    /// A helper to run a generation step on this builder.
    fn apply_generator(&mut self, generator: impl Debug + Generate) {
        timed!(&format!("{:?}", generator), generator.generate(self))
    }
}

/// A type that generates some sort of data for the world. This takes in a set
/// of tiles that have some data generated, and mutates the tiles to add new
/// data. Generally there will be a series of generators chained together,
/// where each one adds some more data until the world is complete.
trait Generate {
    /// Apply some generation step to the given world. This can mutate the
    /// world's tiles, but can never add/remove tiles, or change their positions
    /// in any way. This function _can_ panic, but only because of internal
    /// bugs. Any implementation of this function _should_ be able to handle
    /// any input without returning an error, so any error is considered
    /// exceptional.
    fn generate(&self, world: &mut WorldBuilder);
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
    /// A static pattern that indicates how runoff flows out of this tile. See
    /// [RunoffPattern] for more info. This is only used during world
    /// generation, so it gets thrown away when the full world is built.
    runoff_pattern: Option<RunoffPattern>,
    runoff_traversed: HexDirectionMap<Meter3>,
    features: Vec<GeoFeature>,
}

impl TileBuilder {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            elevation: None,
            rainfall: None,
            runoff: None,
            runoff_pattern: None,
            runoff_traversed: HexDirectionMap::default(),
            biome: None,
            features: Vec::new(),
        }
    }

    /// Finalize this builder to create a [Tile]. Panics if any fields
    /// on this builder are uninitialized.
    pub fn build(self) -> Tile {
        let position = self.position;
        Tile {
            position,
            elevation: self.elevation(),
            rainfall: self.rainfall(),
            runoff: self.runoff(),
            biome: self.biome(),
            features: self.features,
            runoff_traversed: self.runoff_traversed.into(),
        }
    }

    /// See [Tile::elevation]. Panics if elevation is unset.
    pub fn elevation(&self) -> Meter {
        unwrap!(self.elevation, "elevation not initialized for {:?}", self)
    }

    /// Set the elevation for this tile. Panics if the elevation value
    /// is out of [World::ELEVATION_RANGE].
    pub fn set_elevation(&mut self, elevation: Meter) {
        World::ELEVATION_RANGE
            .ensure_contains(elevation)
            .expect("elevation out of range");
        self.elevation = Some(elevation);
    }

    /// See [Tile::rainfall]. Panics if rainfall is unset.
    pub fn rainfall(&self) -> Meter3 {
        unwrap!(self.rainfall, "rainfall not initialized for {:?}", self)
    }

    /// Set the rainfall for this tile. Panics if the given value is
    /// negative.
    pub fn set_rainfall(&mut self, rainfall: Meter3) {
        if rainfall >= Meter3(0.0) {
            self.rainfall = Some(rainfall);
        } else {
            panic!("cannot set negative rainfall {}", rainfall)
        }
    }

    /// See [Tile::humidity]. Panics if humidity is unset.
    pub fn humidity(&self) -> f64 {
        let rainfall = self.rainfall();
        World::RAINFALL_SOFT_RANGE
            .value(rainfall)
            .clamp()
            .convert::<f64>()
            .normalize()
            .inner()
    }

    /// Set the biome for this tile.
    pub fn set_biome(&mut self, biome: Biome) {
        self.biome = Some(biome);
    }

    /// Get a reference to this tile's runoff pattern. The runoff pattern gives
    /// info about how runoff flows out of this tile. Panics if the
    /// runoff pattern is uninitialized.
    pub fn runoff_pattern(&self) -> &RunoffPattern {
        unwrap!(
            self.runoff_pattern.as_ref(),
            "runoff_pattern not initialized for {:?}",
            self
        )
    }

    /// Initialize the runoff pattern for this tile.
    pub fn set_runoff_pattern(&mut self, runoff_pattern: RunoffPattern) {
        self.runoff_pattern = Some(runoff_pattern);
    }

    /// See [Tile::runoff]. Panics if runoff is unset.
    pub fn runoff(&self) -> Meter3 {
        unwrap!(self.runoff, "runoff not initialized for {:?}", self)
    }

    /// Set the runoff level of this tile (any existing runoff will be deleted).
    /// Panics if the runoff value is negative.
    pub fn set_runoff(&mut self, runoff: Meter3) {
        assert!(
            runoff >= Meter3(0.0),
            "cannot set negative runoff {} for {:?}",
            runoff,
            self
        );
        self.runoff = Some(runoff);
    }

    /// Add some amount of runoff from another tile to this one.
    /// `from_direction` indicates which direction the runoff is coming
    /// from, which will be used to track this runoff as ingress.  Returns
    /// an error if the amount is negative or runoff is uninitialized.
    pub fn add_runoff(&mut self, runoff: Meter3, from_direction: HexDirection) {
        assert!(
            runoff >= Meter3(0.0),
            "cannot add negative runoff {} for {:?}",
            runoff,
            self
        );

        self.runoff = Some(self.runoff() + runoff);
        // Track this runoff ingress
        *self.runoff_traversed.entry(from_direction).or_default() += runoff;
    }

    /// Clear all runoff from this tile, and return it in a map that determines
    /// how much runoff to send in each direction. This will always remove
    /// **all** runoff from this tile, and that runoff will be tracked as
    /// egress on this tile. The returned map should be used to add that amount
    /// of runoff to neighboring tiles.
    pub fn distribute_runoff(&mut self) -> HexDirectionMap<Meter3> {
        let distribution =
            self.runoff_pattern().distribute_exits(self.runoff());

        // If we have anywhere to distribute (i.e. if this tile isn't a
        // terminal), then clear our runoff and count it as egress
        if !distribution.is_empty() {
            self.runoff = Some(Meter3(0.0));

            // Track each outgoing chunk of runoff as egress
            for (dir, runoff) in distribution.iter() {
                *self.runoff_traversed.entry(*dir).or_default() -= *runoff;
            }
        }

        distribution
    }

    /// Reset the runoff on this tile to 0 and return whatever amount was here.
    /// Panics if runoff is unset.
    pub fn clear_runoff(&mut self) -> Meter3 {
        let runoff = self.runoff();
        self.runoff = Some(Meter3(0.0));
        runoff
    }

    /// See [Tile::runoff_traversed].
    pub fn runoff_traversed(&self) -> &HexDirectionMap<Meter3> {
        &self.runoff_traversed
    }

    /// See [Tile::biome]. Panics if biome is unset.
    pub fn biome(&self) -> Biome {
        unwrap!(self.biome, "biome not initialized for {:?}", self)
    }

    /// Get this tile's biome as an `Option`, meaning if the biome is unset, we
    /// return `None` (as opposed to [Self::biome], which panics).
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

    /// Add a new geographic feature to this tile. Panics if the tile
    /// already has that feature.
    pub fn add_feature(&mut self, feature: GeoFeature) {
        if !self.features.contains(&feature) {
            self.features.push(feature);
        } else {
            panic!("feature {:?} already exists for {:?}", feature, self)
        }
    }
}

impl HasHexPosition for TileBuilder {
    fn position(&self) -> HexPoint {
        self.position
    }
}
