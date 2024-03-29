mod seed;

use crate::{Meter, Meter3};
use derive_more::Display;
use fnv::FnvHasher;
pub use seed::Seed;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use validator::Validate;

// TODO there's a bunch of fields in here that can't easily be validated
// because they are Meter or Meter3. Maybe a PR to validator that allows
// validating things that are Into<f64>? Or manual validation

/// Configuration that defines a world gen process. Two worlds generated with
/// same config will always be identical.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::system::Resource))]
#[serde(default)]
pub struct WorldConfig {
    /// RNG seed to use for all randomized processes during world gen. See the
    /// [Seed] type for details on the different values supported here.
    pub seed: Seed,

    /// Distance from the center of the world to the edge (in tiles).
    #[validate(range(min = 0, max = 10000))]
    pub radius: u16,

    /// Config for the noise function used to generate elevation values
    #[validate]
    pub elevation: ElevationConfig,

    /// Config for fields related to rainfall and evaporation
    #[validate]
    pub rainfall: RainfallConfig,

    /// Config for fields related to geographic feature generation
    #[validate]
    pub geo_feature: GeoFeatureConfig,
}

/// Configuration for elevation map generation. This controls the elevation of
/// each tile, which defines the shape of the terrain. Elevation is generated
/// by a noise function, then some post-processing is applied.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct ElevationConfig {
    /// Configuration for the noise function used to generate elevation values
    pub noise_fn: NoiseFnConfig,

    /// If defined, each elevation value will be rounded to the nearest
    /// multiple of this interval. E.g. if the interval is 10, each
    /// elevation will be rounded to the nearest 10 meters.
    ///
    /// This supports any positive number, including fractions
    // TODO validate >0
    pub rounding_interval: Option<Meter>,

    /// The fraction of the world's radius that is buffer space. Tiles in the
    /// buffer space will be pushed down, to ensure that the very edge of the
    /// world is all ocean. The closer to the edge a tile is, the more it will
    /// be pushed. 1.0 means the world is _entirely_ buffer space, 0.0 means
    /// there is no buffer at all, 0.25 means the outer 25% is buffer, etc.
    #[validate(range(min = 0.0, max = 1.0))]
    pub edge_buffer_fraction: f64,

    /// Exponent to apply to the function that pushes down elevations in the
    /// buffer zone. An exponent of 1.0 will push them linearly. Sub-1.0
    /// exponents will have a gradual dropoff closer to the middle, then get
    /// steeper towards the edge. Super-1.0 exponents will do the opposite
    /// (steep at first, then gradual out at the edge).
    pub edge_buffer_exponent: f64,
}

/// Configuration related to rainfall and evaporation simulation. These params
/// control how rainfall is generated for the world, which in turn has a major
/// impact on runoff and feature generation.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct RainfallConfig {
    /// Should rainfall and runoff be simulated? Rainfall and runoff simulation
    /// are fairly slow, so disable this if you need quick generation and don't
    /// need rainfall or runoff. Useful for iterating on other things.
    pub enabled: bool,

    /// The amount of evaporation that each tile provides under "default"
    /// circumstances. ATM "default" means water, but that could be subject to
    /// change. In general though, this is the base evaporation value we use,
    /// and it can be modified under certain scenarios according to other
    /// fields in this config.
    pub evaporation_default: Meter3,

    /// Scaling factor for evaporation from land tiles. Each land tile will
    /// produce the default evaporation amount times this scaling factor.
    /// Should probably be less than 1.
    #[validate(range(min = 0.0))]
    pub evaporation_land_scale: f64,

    /// The distance (in tiles) that evaporation spreads, perpendicular to the
    /// wind. E.g. if we consider the wind direction to be *forward*, then
    /// this is the distance to the left and right that a particular tile's
    /// evaporation will spread. This is a smoothing mechanism that makes
    /// precipitation patterns appear smoother/more natural.
    #[validate(range(min = 0))]
    pub evaporation_spread_distance: u16,

    /// Exponent to apply while calculating spread diminishment. If the
    /// exponent is 1.0, then evaporation spread will be linear, meaning the
    /// amount of evaporation that one tile will receive from another tile that
    /// is `n` steps away will be proportional to `n`. If this is <1, then
    /// spreading will be biased towards the center, and if >1 will be biased
    /// towards the outer edges.
    pub evaporation_spread_exponent: f64,

    /// The maximum fraction of a cloud's rainfall that can be dropped on any
    /// particular tile. E.g. if this is 0.01, then a cloud can drop at most 1%
    /// of its held water on a single tile. This value should typically be
    /// pretty small, to allow water spreading over large chunks of land.
    #[validate(range(min = 0.0, max = 1.0))]
    pub rainfall_fraction_limit: f64,
}

/// Configuration surrounding how geographic features are generated. See
/// [GeoFeature](crate::GeoFeature) for more info.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct GeoFeatureConfig {
    /// The minimum amount of runoff that must enter or exit a tile in a single
    /// direction for that direction to be considered a river. Unlike
    /// `lake_runoff_threshold`, this tracks total _traversed_ runoff, not
    /// _collected_ runoff. That means it pertains to the amount of runoff that
    /// passed over a tile (from/towards a particular direction) as opposed to
    /// the runoff that ended up on the tile after runoff simulation finished.
    pub river_runoff_traversed_threshold: Meter3,
}

/// Config for a particular noise generation function. We use
/// https://crates.io/crates/noise for noise generation. This type is generic,
/// i.e. not specific to a particular noise function, so as such it has no
/// default implementation.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct NoiseFnConfig {
    pub noise_type: NoiseFnType,

    /// Number of different frequencies to add together. We can use multiple
    /// octaves to build a set of curves, then add them together to get our
    /// final function.
    #[validate(range(min = 0))]
    pub octaves: usize,

    /// The frequency of the first (lowest) octave.
    #[validate(range(min = 0.01))]
    pub frequency: f64,

    /// Constant to add to the frequency for each octave. E.g. if we have 3
    /// octaves, a base frequency of 1.0, and a lacunarity of 2.0, then our
    /// 3 octaves will be at [1.0, 3.0, 5.0].
    pub lacunarity: f64,

    /// Amplification factor for each octave. The first amplitude is always
    /// 1.0, then is multiplied by the persistence for each octave. E.g. with 3
    /// octaves and a persistence of 0.5, your amplitudes will be `[1.0, 0.5,
    /// 0.25]`.
    pub persistence: f64,

    /// Exponent to apply to elevation values after generation. This is applied
    /// to normalized composite values. "Normalized" means they're in the
    /// range [0,1] (meaning we can apply any exponent and the values
    /// remain in that range) and "composite" means this is *after* we add
    /// all our octaves together. Exponents <1 bias upwards, and >1 bias
    /// downwards.
    pub exponent: f64,
}

/// The different types of supported noise functions. These are all expected to
/// be seedable and multi-fractal. See
/// https://docs.rs/noise/0.7.0/noise/trait.MultiFractal.html for a list of
/// types that could possibly be supported here.
#[derive(
    Copy, Clone, Debug, Display, Eq, PartialEq, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum NoiseFnType {
    // if you add a variant here, make sure you update the type in wasm/lib.rs
    #[display(fmt = "Basic Multi")]
    BasicMulti,
    Billow,
    #[display(fmt = "Fractal Brownian Motion")]
    Fbm,
    #[display(fmt = "Hybrid Multi")]
    HybridMulti,
    #[display(fmt = "Ridged Multi")]
    RidgedMulti,
}

impl WorldConfig {
    /// Convert a string to a numeric value that can be used as a 64-bit RNG
    /// seed in a world config. This will attempt to parse the string as a
    /// number. If that fails, it will hash the string into a number.
    pub fn str_to_seed(seed_str: &str) -> u64 {
        seed_str.parse::<u64>().unwrap_or_else(|_| {
            let mut hasher = FnvHasher::default();
            seed_str.hash(&mut hasher);
            hasher.finish()
        })
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        // This should be the general source of truth for a "nice world", but
        // doesn't need to be kept 100% up to date. But we want to make sure
        // that whenever someone generates a world with the default config,
        // it looks pretty good.
        Self {
            seed: Default::default(),
            radius: 100,
            elevation: Default::default(),
            rainfall: Default::default(),
            geo_feature: Default::default(),
        }
    }
}

impl Default for Seed {
    fn default() -> Self {
        // Danger! This means the default will vary between calls!
        Self::Int(rand::random())
    }
}

impl Default for ElevationConfig {
    fn default() -> Self {
        Self {
            noise_fn: NoiseFnConfig {
                noise_type: NoiseFnType::Fbm,
                octaves: 3,
                frequency: 0.5,
                lacunarity: 3.0,
                persistence: 0.3,
                exponent: 0.9,
            },
            rounding_interval: None,
            edge_buffer_fraction: 0.25,
            edge_buffer_exponent: 0.7,
        }
    }
}

impl Default for RainfallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            evaporation_default: Meter3(5.0),
            evaporation_land_scale: 0.35,
            evaporation_spread_distance: 50,
            evaporation_spread_exponent: 0.6,
            rainfall_fraction_limit: 0.03,
        }
    }
}

impl Default for GeoFeatureConfig {
    fn default() -> Self {
        Self {
            river_runoff_traversed_threshold: Meter3(100.0),
        }
    }
}
