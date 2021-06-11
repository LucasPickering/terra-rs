use crate::{
    config::NoiseFnType, util::range::Rangeable, HexPoint, NoiseFnConfig,
    NumRange, RangeValue,
};
use noise::{Fbm, MultiFractal, NoiseFn, RidgedMulti, Seedable};
use rand::Rng;
use std::fmt::Debug;

/// Helper trait for the different types of noise functions we use. We need this
/// in order to create trait objects.
trait NoiseFnTrait: Debug + NoiseFn<[f64; 3]> {}

impl<T: Debug + NoiseFn<[f64; 3]>> NoiseFnTrait for T {}

/// A wrapper around a noise function that makes it easy to use for generating
/// tile values. This is initialized for a particular function type, and
/// makes it easy to pass in a [HexPoint] and get out values in an arbitrary
/// output range.
///
/// This type can optionally also do transparent conversions on the output type,
/// e.g. if you are using a newtype that wraps `f64`.
#[derive(Debug)]
pub struct TileNoiseFn<T: Rangeable<f64> = f64> {
    /// The noise generation function
    noise_fn: Box<dyn NoiseFnTrait>,
    /// The config that defines the behavior of this function
    config: NoiseFnConfig,
    /// The range of values that outputs should be mapped to. Generally the
    /// output will span this entire range, but it may not. This depends on
    /// the behavior of the underlying noise function.
    output_range: NumRange<T, f64>,
}

impl<T> TileNoiseFn<T>
where
    T: Rangeable<f64>,
    f64: From<T>,
{
    /// If we used the full values from the input, our frequencies would have
    /// to be stupid low to get resonable looking output, so we scale them
    /// down by this factor
    const INPUT_SCALE: f64 = 100.0;
    /// The output range of the internal noise function. Used to map the noise
    /// values to our own output range.
    const NOISE_FN_OUTPUT_RANGE: NumRange<f64> = NumRange::new(-1.0, 1.0);

    /// Get the function output at the given point
    pub fn get(&self, point: HexPoint) -> RangeValue<T, f64> {
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
            .apply(|val| val.powf(self.config.exponent))
            // Convert to type T so we can map to the output range
            .convert() // f64 -> T
            .map_to(self.output_range)
    }
}

impl<T: Rangeable<f64>> TileNoiseFn<T> {
    /// Initialize a new function for some underlying noise fn type.
    ///
    /// ### Arguments
    /// - `world_config` - The overall world config, needed for seed and world
    /// radius.
    /// - `config` - Configuration for the underlying noise function.
    /// - `output_range` - The output range of this function. Noise values will
    /// be mapped to this range during generation.
    pub fn new(
        rng: &mut impl Rng,
        config: NoiseFnConfig,
        output_range: NumRange<T, f64>,
    ) -> Self {
        // Gen a new seed so that we get a different one per function
        let seed = rng.gen();
        let noise_fn = Self::make_noise_fn(config, seed);

        Self {
            noise_fn,
            config,
            output_range,
        }
    }

    /// Create a noise function based on this function config. Since the config
    /// contains the function type, we don't know which struct the function will
    /// be at compile time, so we need a trait object.
    fn make_noise_fn(
        config: NoiseFnConfig,
        seed: u32,
    ) -> Box<dyn NoiseFnTrait> {
        // Seedable and MultiFractal can't be turned into trait objects, so
        // we can't use dynamic dispatch to configure the function. That means
        // we have to duplicate the configuration code for each function type,
        // which sucks but 'tis what 'tis.
        match config.noise_type {
            NoiseFnType::BasicMulti => Box::new(
                Fbm::default()
                    .set_seed(seed)
                    .set_octaves(config.octaves)
                    .set_frequency(config.frequency)
                    .set_lacunarity(config.lacunarity)
                    .set_persistence(config.persistence),
            ),
            NoiseFnType::Billow => Box::new(
                Fbm::default()
                    .set_seed(seed)
                    .set_octaves(config.octaves)
                    .set_frequency(config.frequency)
                    .set_lacunarity(config.lacunarity)
                    .set_persistence(config.persistence),
            ),
            NoiseFnType::Fbm => Box::new(
                Fbm::default()
                    .set_seed(seed)
                    .set_octaves(config.octaves)
                    .set_frequency(config.frequency)
                    .set_lacunarity(config.lacunarity)
                    .set_persistence(config.persistence),
            ),
            NoiseFnType::HybridMulti => Box::new(
                Fbm::default()
                    .set_seed(seed)
                    .set_octaves(config.octaves)
                    .set_frequency(config.frequency)
                    .set_lacunarity(config.lacunarity)
                    .set_persistence(config.persistence),
            ),
            NoiseFnType::RidgedMulti => Box::new(
                RidgedMulti::default()
                    .set_seed(seed)
                    .set_octaves(config.octaves)
                    .set_frequency(config.frequency)
                    .set_lacunarity(config.lacunarity)
                    .set_persistence(config.persistence),
            ),
        }
    }
}
