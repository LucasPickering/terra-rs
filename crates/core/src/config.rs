use serde::{Deserialize, Serialize};

use crate::Meter3;

/// Config for a particular noise generation function
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NoiseFnConfig {
    /// Number of different frequencies to add together. We can use multiple
    /// octaves to build a set of curves, then add them together to get our
    /// final function.
    pub octaves: usize,

    /// The frequency of the first (lowest) octave.
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

/// Configuration that defines a world gen process. Two worlds generated with
/// same config will always be identical.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WorldConfig {
    /// RNG seed to use for all randomized processes during world gen.
    #[serde(default = "rand::random")]
    pub seed: u64,

    /// Distance from the center of the world to the edge (in tiles).
    pub radius: u16,

    /// Buffer space at the edge of the world where we gradually push
    /// elevations down, to ensure that the very edge of the world is all
    /// ocean. This is included **as part of the radius**, and therefore must
    /// be less than the radius.
    pub edge_buffer_size: u16,
    /// Exponent to apply to the function that pushes down elevations in the
    /// buffer zone. An exponent of 1.0 will push them linearly. Sub-1.0
    /// exponents will have a smooth dropoff closer to the middle, then get
    /// steeper towards the edge. Super-1.0 exponents will do the opposite
    /// (steep at first, then smooth out at the edge).
    pub edge_buffer_exponent: f64,

    /// The amount of evaporation that each tile provides under "default"
    /// circumstances. ATM "default" means water, but that could be subject to
    /// change. In general though, this is the base evaporation value we use,
    /// and it can be modified under certain scenarios according to other
    /// fields in this config.
    pub evaporation_default: Meter3,
    /// Scaling factor for evaporation from land tiles. Each land tile will
    /// produce the default evaporation amount times this scaling factor.
    /// Should probably be less than 1.
    pub evaporation_land_scale: f64,
    /// The distance (in tiles) that evaporation spreads, perpendicular to the
    /// wind. E.g. if we consider the wind direction to be *forward*, then
    /// this is the distance to the left and right that a particular tile's
    /// evaporation will spread. This is a smoothing mechanism that makes
    /// precipitation patterns appear smoother/more natural.
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
    pub rainfall_factor_limit: f64,

    /// Config for the noise function used to generate elevation values
    pub elevation: NoiseFnConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        // This should be the general source of truth for a "nice world", but
        // doesn't need to be kept 100% up to date. But we want to make sure
        // that whenever someone generates a world with the default config,
        // it looks pretty good.
        WorldConfig {
            seed: 0, // You'll probably want to override this with rand::random
            radius: 100,
            edge_buffer_size: 25,
            edge_buffer_exponent: 0.7,
            evaporation_default: Meter3(3.0),
            evaporation_land_scale: 0.22,
            evaporation_spread_distance: 50,
            evaporation_spread_exponent: 0.6,
            rainfall_factor_limit: 0.03,
            elevation: NoiseFnConfig {
                octaves: 3,
                frequency: 0.5,
                lacunarity: 3.0,
                persistence: 0.3,
                exponent: 0.9,
            },
        }
    }
}
