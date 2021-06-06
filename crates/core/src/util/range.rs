use crate::{Meter, Meter2, Meter3};
use anyhow::anyhow;
use derive_more::Display;
use rand::{
    distributions::uniform::{SampleRange, SampleUniform, UniformSampler},
    RngCore,
};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops,
};

/// A type of value that we can create ranges of, where a range has a min and
/// max. This allows us to do all kinds of neat conversions and shit. Usually,
/// the type parameter `I` isn't necessary, because it's just `Self`. It's
/// useful in some situations though where you want to have ranges of
/// non-numeric types, e.g. a newtype that holds an `f64`. In that case, the
/// type param would be whatever internal type you use for the math.
pub trait Rangeable<I = Self>:
    Copy
    + Debug
    + Display
    + PartialOrd
    + From<I>
    + Into<I>
    + ops::Add<Self, Output = Self>
    + ops::Sub<Self, Output = Self>
    + ops::Mul<I, Output = Self>
    + ops::Div<I, Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
}

impl Rangeable for f32 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }
}

impl Rangeable for f64 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }
}

impl Rangeable<f64> for Meter {
    fn zero() -> Self {
        0.0.into()
    }

    fn one() -> Self {
        1.0.into()
    }
}

impl Rangeable<f64> for Meter2 {
    fn zero() -> Self {
        0.0.into()
    }

    fn one() -> Self {
        1.0.into()
    }
}

impl Rangeable<f64> for Meter3 {
    fn zero() -> Self {
        0.0.into()
    }

    fn one() -> Self {
        1.0.into()
    }
}

/// A range between two numeric values, inclusive on both ends.
///
/// ## Type Parameters
///
/// - `T` represents the type of values represented by this range, e.g. `f64` or
///   `Meter`
/// - `I` represents the underlying primitive type that we use for numberic
///   comparisons. E.g. for `Meter` we map down to `f64`, but for `f64` it's
///   still just `f64`
#[derive(Copy, Clone, Debug, Display, PartialEq)]
#[display(fmt = "[{}, {}]", min, max)]
pub struct NumRange<T: Rangeable<I>, I = T> {
    pub min: T,
    pub max: T,
    phantom: PhantomData<I>,
}

impl<T: Into<I> + Rangeable<I>, I> NumRange<T, I> {
    pub const fn new(min: T, max: T) -> Self {
        Self {
            min,
            max,
            phantom: PhantomData,
        }
    }

    /// Get a [0,1] range for this type.
    pub fn normal_range() -> Self {
        Self::new(T::zero(), T::one())
    }

    /// Create a [RangeValue] in this range, which is convenient for chaining
    ///  operations on a single value.
    pub fn value(self, value: T) -> RangeValue<T, I> {
        RangeValue { value, range: self }
    }
    /// Max minus min
    pub fn span(&self) -> T {
        self.max - self.min
    }

    /// Create a new range that has the same span (max-min) as this range, but
    /// the minimum value is zero.
    pub fn zeroed(&self) -> Self {
        Self::new(T::zero(), self.span())
    }

    /// Check if a value is in this range. Ranges are inclusive on both ends.
    pub fn contains(&self, value: T) -> bool {
        self.min <= value && value <= self.max
    }

    /// Checks if the value is in this range. If it isn't, return an error.
    pub fn ensure_contains(&self, value: T) -> anyhow::Result<()> {
        if self.contains(value) {
            Ok(())
        } else {
            Err(anyhow!("value {} is not in range {}", value, self))
        }
    }

    /// Map a value from this range to the target range. If the span of this
    /// range is zero, we can't properly map the value because we don't know
    /// where on the target range it should fall. In that case, we just always
    /// return the **minimum** of the target range.
    pub fn map_to(&self, dest_range: &Self, value: T) -> T {
        let span = self.span();
        if span > T::zero() {
            // Map down to [0,1], then map back up to the target range
            let normalized = (value - self.min) / span.into();
            dest_range.min + (normalized * dest_range.span().into())
        } else {
            // Source span is zero, so we can't do a proper mapping (which would
            // just return NaN). Arbitrarily pick the min bound on the target
            dest_range.min
        }
    }

    /// Map a value from this range to the range [0, 1]
    pub fn normalize(&self, value: T) -> T {
        self.map_to(&Self::normal_range(), value)
    }

    /// Force a value into this range. If it's already in the range, return
    /// that value. If it's outside the range, return the bound (lower or upper)
    /// that's closest to the value.
    pub fn clamp(&self, value: T) -> T {
        if value < self.min {
            self.min
        } else if value > self.max {
            self.max
        } else {
            value
        }
    }
}

// allow generating samples in the range
impl<T: Rangeable + SampleUniform + PartialOrd> SampleRange<T> for NumRange<T> {
    #[inline]
    fn sample_single<R: RngCore + ?Sized>(self, rng: &mut R) -> T {
        T::Sampler::sample_single_inclusive(self.min, self.max, rng)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.min > self.max
    }
}

/// An alternative interface for [NumRange] that makes it easy to chain
/// operations on a single value.
///
/// ```
/// use terra::NumRange;
///
/// let range: NumRange<f32> = NumRange::new(10.0, 20.0);
/// let value = range.value(15.0).normalize().apply(|x| x + 1.0).inner();
/// assert_eq!(value, 1.5);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct RangeValue<T: Rangeable<I>, I> {
    value: T,
    range: NumRange<T, I>,
}

impl<T: Into<I> + Rangeable<I>, I: Debug> RangeValue<T, I> {
    /// Get the value from this struct
    pub fn inner(self) -> T {
        self.value
    }

    /// Print out debug info on this range. Only enabled for debug builds.
    #[cfg(debug_assertions)]
    pub fn dbg(self) -> Self {
        dbg!(self)
    }

    /// Map this value to the range [0,1]
    pub fn normalize(self) -> Self {
        self.map_to(<NumRange<T, I>>::normal_range())
    }

    /// Invert this value in the range, so that its distance from the min
    /// becomes its distance from the max, and vice versa. For example,
    /// inverting `0.7` in the range `[0,1]` returns `0.3`.
    pub fn invert(mut self) -> Self {
        let min = self.range.max;
        let max = self.range.min;
        self.value = self.range.map_to(&NumRange::new(min, max), self.value);
        self
    }

    /// Map this value from the current range to a new range.
    pub fn map_to(self, range: NumRange<T, I>) -> Self {
        let new_value = self.range.map_to(&range, self.value);
        Self {
            range,
            value: new_value,
        }
    }

    /// Force the given value into this range. If it falls outside the range,
    /// it will be set to the nearer of the two bounds. In other words, if it's
    /// below the range, use the range min. If it's above the range, use the
    /// max.
    pub fn clamp(self) -> Self {
        Self {
            value: self.range.clamp(self.value),
            range: self.range,
        }
    }

    /// Apply the given mapping function to this value. The value will be
    /// replaced with the output of the function.
    pub fn apply(self, f: impl FnOnce(T) -> T) -> Self {
        Self {
            value: f(self.value),
            range: self.range,
        }
    }

    /// Convert this value to another type with a transparent conversion. It
    /// would be nice to just provide this as a `From` implementation, but that
    /// gets conflicts with std's blanket implementation, so it's not possible
    /// until specialization is done.
    pub fn convert<U: Rangeable<I> + From<T>>(self) -> RangeValue<U, I> {
        let value = U::from(self.value);
        let range =
            NumRange::new(U::from(self.range.min), U::from(self.range.max));
        RangeValue { value, range }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_normal_range() {
        let range: NumRange<f64> = NumRange::normal_range();
        assert_approx_eq!(range.min, 0.0);
        assert_approx_eq!(range.max, 1.0);
    }

    #[test]
    fn test_span() {
        let range: NumRange<f64> = NumRange::new(1.0, 3.0);
        assert_approx_eq!(range.span(), 2.0);
    }

    #[test]
    fn test_zeroed() {
        assert_eq!(NumRange::new(1.0, 3.0).zeroed(), NumRange::new(0.0, 2.0));
        assert_eq!(NumRange::new(-3.0, -1.0).zeroed(), NumRange::new(0.0, 2.0));
    }

    #[test]
    fn test_contains() {
        let range: NumRange<f64> = NumRange::new(1.0, 3.0);
        assert!(!range.contains(0.9));
        assert!(range.contains(1.0));
        assert!(range.contains(2.0));
        assert!(range.contains(3.0));
        assert!(!range.contains(3.1));

        // Test a zero-length span, it should contain exactly one value
        let range: NumRange<f64> = NumRange::new(1.0, 1.0);
        assert!(!range.contains(0.9));
        assert!(range.contains(1.0));
        assert!(!range.contains(1.1));
    }

    #[test]
    fn test_map_to() {
        let input_range: NumRange<f64> = NumRange::new(1.0, 3.0);
        let output_range: NumRange<f64> = NumRange::new(20.0, 40.0);
        assert_approx_eq!(input_range.map_to(&output_range, 0.0), 10.0);
        assert_approx_eq!(input_range.map_to(&output_range, 1.0), 20.0);
        assert_approx_eq!(input_range.map_to(&output_range, 2.0), 30.0);
        assert_approx_eq!(input_range.map_to(&output_range, 3.0), 40.0);
        assert_approx_eq!(input_range.map_to(&output_range, 6.0), 70.0);

        // Test a zero-length span, it should always map to the min of the
        // output range
        let input_range: NumRange<f64> = NumRange::new(1.0, 1.0);
        assert_approx_eq!(input_range.map_to(&output_range, 0.0), 20.0);
        assert_approx_eq!(input_range.map_to(&output_range, 1.0), 20.0);
        assert_approx_eq!(input_range.map_to(&output_range, 1.5), 20.0);
    }

    #[test]
    fn test_normalize() {
        let range: NumRange<f64> = NumRange::new(1.0, 3.0);
        assert_approx_eq!(range.normalize(0.0), -0.5);
        assert_approx_eq!(range.normalize(1.0), 0.0);
        assert_approx_eq!(range.normalize(2.0), 0.5);
        assert_approx_eq!(range.normalize(3.0), 1.0);
        assert_approx_eq!(range.normalize(6.0), 2.5);

        // Test a zero-length span, it should always map to zero
        let range: NumRange<f64> = NumRange::new(1.0, 1.0);
        assert_approx_eq!(range.normalize(0.0), 0.0);
        assert_approx_eq!(range.normalize(1.0), 0.0);
        assert_approx_eq!(range.normalize(1.5), 0.0);
    }

    #[test]
    fn test_clamp() {
        let range: NumRange<f64> = NumRange::new(1.0, 3.0);
        assert_approx_eq!(range.clamp(0.0), 1.0);
        assert_approx_eq!(range.clamp(1.0), 1.0);
        assert_approx_eq!(range.clamp(2.0), 2.0);
        assert_approx_eq!(range.clamp(3.0), 3.0);
        assert_approx_eq!(range.clamp(6.0), 3.0);

        // Test a zero-length span, it should always map to the same value
        let range: NumRange<f64> = NumRange::new(1.0, 1.0);
        assert_approx_eq!(range.clamp(0.0), 1.0);
        assert_approx_eq!(range.clamp(1.0), 1.0);
        assert_approx_eq!(range.clamp(1.5), 1.0);
    }
}
