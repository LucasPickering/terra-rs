use crate::Meter3;
use serde::{Deserialize, Serialize};
use validator::Validate;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Configuration that defines a world gen process. Two worlds generated with
/// same config will always be identical.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct WorldConfig {
    /// RNG seed to use for all randomized processes during world gen.
    ///
    /// When deserializing a config, this field supports a few options:
    /// - If the value is an integer that fits into `u64`, use that value
    /// - If it's a string that can be parsed into a `u64`, use the parsed
    ///   value
    /// - If it's_any other string, hash it and use the hash value
    /// - If it's anything else (out of range number, float, array, etc.),
    ///   error
    ///
    /// **Note:** It seems that some serde implementations (including
    /// serde_json) will be overzealous and accidentally support additional
    /// data types here. E.g. if you pass a bool, it will stringify it then
    /// hash the string. Don't consider that supported behavior, just a
    /// bug.
    ///
    /// Regardless of how the seed value is input, it will always be serialized
    /// as a number.
    #[serde(deserialize_with = "serde_seed::deserialize")]
    pub seed: u64,

    /// Distance from the center of the world to the edge (in tiles).
    #[validate(range(min = 0, max = 10000))]
    pub radius: u16,

    /// The fraction of the world's radius that is buffer space. Tiles in the
    /// buffer space will be pushed down, to ensure that the very edge of the
    /// world is all ocean. The closer to the edge a tile is, the more it will
    /// be pushed. 1.0 means the world is _entirely_ buffer space, 0.0 means
    /// there is no buffer at all, 0.25 means the outer 25% is buffer, etc.
    #[validate(range(min = 0.0, max = 1.0))]
    pub edge_buffer_fraction: f64,
    /// Exponent to apply to the function that pushes down elevations in the
    /// buffer zone. An exponent of 1.0 will push them linearly. Sub-1.0
    /// exponents will have a smooth dropoff closer to the middle, then get
    /// steeper towards the edge. Super-1.0 exponents will do the opposite
    /// (steep at first, then smooth out at the edge).
    pub edge_buffer_exponent: f64,

    /// Config for fields related to rainfall and evaporation
    #[validate]
    pub rainfall: RainfallConfig,

    /// Config for fields related to geographic feature generation
    #[validate]
    pub geo_feature: GeoFeatureConfig,

    /// Config for the noise function used to generate elevation values
    #[validate]
    pub elevation: NoiseFnConfig,
}

/// Configuration related to rainfall and evaporation simulation. These params
/// control how rainfall is generated for the world, which in turn has a major
/// impact on runoff and feature generation.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct RainfallConfig {
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct GeoFeatureConfig {
    /// The minimum amount of runoff that needs to collect on a tile for it to
    /// become a lake. Any tile with this amount of runoff or more on it after
    /// runoff simulation will become a lake. Any tile with less will not.
    pub lake_runoff_threshold: Meter3,

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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Validate)]
pub struct NoiseFnConfig {
    pub noise_type: NoiseFnType,

    /// Number of different frequencies to add together. We can use multiple
    /// octaves to build a set of curves, then add them together to get our
    /// final function.
    #[validate(range(min = 0))]
    pub octaves: usize,

    /// The frequency of the first (lowest) octave.
    #[validate(range(min = 0.0))]
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

    /// Exponent to apply to values after generation. This is applied to
    /// normalized composite values. "Normalized" means they're in the range
    /// [0,1] (meaning we can apply any exponent and the values remain in that
    /// range) and "composite" means this is *after* we add all our octaves
    /// together.
    pub exponent: f64,
}

/// The different types of supported noise functions. These are all expected to
/// be seedable and multi-fractal. See
/// https://docs.rs/noise/0.7.0/noise/trait.MultiFractal.html for a list of
/// types that could possibly be supported here.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseFnType {
    // if you add a variant here, make sure you update the type in wasm/lib.rs
    BasicMulti,
    Billow,
    Fbm,
    HybridMulti,
    RidgedMulti,
}

impl Default for WorldConfig {
    fn default() -> Self {
        // This should be the general source of truth for a "nice world", but
        // doesn't need to be kept 100% up to date. But we want to make sure
        // that whenever someone generates a world with the default config,
        // it looks pretty good.
        Self {
            // Danger! This means the default will vary between calls!
            seed: rand::random(),

            radius: 100,
            edge_buffer_fraction: 0.25,
            edge_buffer_exponent: 0.7,
            rainfall: RainfallConfig::default(),
            geo_feature: GeoFeatureConfig::default(),
            elevation: NoiseFnConfig {
                noise_type: NoiseFnType::Fbm,
                octaves: 3,
                frequency: 0.5,
                lacunarity: 3.0,
                persistence: 0.3,
                exponent: 0.9,
            },
        }
    }
}

impl Default for RainfallConfig {
    fn default() -> Self {
        Self {
            evaporation_default: Meter3(3.0),
            evaporation_land_scale: 0.2,
            evaporation_spread_distance: 50,
            evaporation_spread_exponent: 0.6,
            rainfall_fraction_limit: 0.03,
        }
    }
}

impl Default for GeoFeatureConfig {
    fn default() -> Self {
        Self {
            lake_runoff_threshold: Meter3(10.0),
            river_runoff_traversed_threshold: Meter3(100.0),
        }
    }
}

/// The seed field has some fancy deserialization behavior implemented here. See
/// the `seed` field definition for a description.
mod serde_seed {
    use fnv::FnvHasher;
    use serde::{de::Visitor, Deserializer};
    use std::{
        convert::TryInto,
        fmt,
        hash::{Hash, Hasher},
    };

    /// Macro to make it easier to implement visit logic for different types
    macro_rules! impl_visit {
        ($fname:ident, $type:ty) => {
            fn $fname<E>(self, value: $type) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.try_into().map_err(|_| {
                    E::custom(format!("u64 out of range: {}", value))
                })
            }
        };
    }

    struct SeedVisitor;

    impl<'de> Visitor<'de> for SeedVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or string")
        }

        // yay for metaprogramming
        impl_visit!(visit_u8, u8);
        impl_visit!(visit_u16, u16);
        impl_visit!(visit_u32, u32);
        impl_visit!(visit_u64, u64);
        impl_visit!(visit_u128, u128);
        impl_visit!(visit_i8, i8);
        impl_visit!(visit_i16, i16);
        impl_visit!(visit_i32, i32);
        impl_visit!(visit_i64, i64);
        impl_visit!(visit_i128, i128);

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value.parse::<u64>() {
                Ok(seed) => Ok(seed),
                Err(_) => {
                    let mut hasher = FnvHasher::default();
                    value.hash(&mut hasher);
                    Ok(hasher.finish())
                }
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We can deserialize from a bunch of different types so we can't give
        // a type hint here
        deserializer.deserialize_any(SeedVisitor)
    }
}
