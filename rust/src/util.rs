use anyhow::anyhow;
use log::debug;
use rand::{
    distributions::uniform::{SampleRange, SampleUniform, UniformSampler},
    RngCore,
};
use serde::Serialize;
use std::{
    fmt::{Debug, Display},
    ops,
};
use wasm_bindgen::prelude::*;

/// A macro to measure the evaluation time of an expression. Wraps an
/// expression, and outputs a tuple of the value of the expression with the
/// elapsed time, as a [Duration](std::time::Duration).
#[macro_export]
macro_rules! timed {
    ($ex:expr) => {{
        use std::time::Duration;

        // https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
        let perf = web_sys::window()
            .expect("should have a Window")
            .performance()
            .expect("should have a Performance");
        let start = perf.now();
        let value = $ex;
        let elapsed = perf.now() - start;
        let elapsed = Duration::from_secs_f64(elapsed / 1000.0);
        (value, elapsed)
    }};
}

/// An RGB color. Values are stored as floats between 0 and 1 (inclusive).
/// This uses f32 because the extra precision from f64 is pointless.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Color3 {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color3 {
    /// The valid range of values for each component in RGB
    const COMPONENT_RANGE: NumRange<f32> = NumRange::new(0.0, 1.0);

    /// Create a new RGB color. Return if any of the components are out of
    /// the range [0.0, 1.0].
    pub fn new(red: f32, green: f32, blue: f32) -> anyhow::Result<Self> {
        fn check_component(
            component_name: &str,
            value: f32,
        ) -> anyhow::Result<f32> {
            if Color3::COMPONENT_RANGE.contains(value) {
                Ok(value)
            } else {
                Err(anyhow!(
                    "Color component {} must be in {}, but was {}",
                    component_name,
                    Color3::COMPONENT_RANGE,
                    value
                ))
            }
        }

        Ok(Self {
            red: check_component("red", red)?,
            green: check_component("green", green)?,
            blue: check_component("blue", blue)?,
        })
    }
}

// Scale a color by a constant
impl ops::Mul<f32> for Color3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let red = Self::COMPONENT_RANGE.clamp(self.red * rhs);
        let green = Self::COMPONENT_RANGE.clamp(self.green * rhs);
        let blue = Self::COMPONENT_RANGE.clamp(self.blue * rhs);
        Self::new(red, green, blue).unwrap()
    }
}

pub trait Rangeable:
    Copy
    + Debug
    + PartialOrd
    + ops::Add<Self, Output = Self>
    + ops::Sub<Self, Output = Self>
    + ops::Mul<Self, Output = Self>
    + ops::Div<Self, Output = Self>
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

/// A range between two numeric values, inclusive on both ends.
#[derive(Copy, Clone, Debug)]
pub struct NumRange<T: Rangeable> {
    pub min: T,
    pub max: T,
}

impl<T: Rangeable> NumRange<T> {
    pub const fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    /// Get a [0,1] range for this type.
    pub fn normal_range() -> Self {
        Self::new(T::zero(), T::one())
    }

    /// Max minus min
    pub fn span(&self) -> T {
        self.max - self.min
    }

    /// Create a [RangeValue] in this range, which can be more convenient for
    /// chaining operations.
    pub fn value(self, value: T) -> RangeValue<T> {
        RangeValue { value, range: self }
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

    /// Map a value from this range to the target range.
    pub fn map(&self, dest_range: &Self, value: T) -> T {
        let normalized = (value - self.min) / self.span();
        dest_range.min + (normalized * dest_range.span())
    }

    /// Map a value from this range to the range [0, 1]
    pub fn normalize(&self, value: T) -> T {
        self.map(&Self::normal_range(), value)
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

// pretty print!
impl<T: Rangeable + Display> Display for NumRange<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
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
/// ```
/// let range: NumRange<f32> = NumRange::new(10.0, 20.0);
/// let value = range.value(15.0).normalize().apply(|x| x + 1.0).inner();
/// assert_eq!(value, 1.5);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct RangeValue<T: Rangeable> {
    value: T,
    range: NumRange<T>,
}

impl<T: Rangeable> RangeValue<T> {
    /// Get the value from this struct
    pub fn inner(self) -> T {
        self.value
    }

    pub fn debug(self) -> Self {
        debug!("{:?}", self.value);
        self
    }

    /// Map this value to the range [0,1]
    pub fn normalize(self) -> Self {
        self.map_to(<NumRange<T>>::normal_range())
    }

    /// Map this value from the current range to a new range.
    pub fn map_to(self, range: NumRange<T>) -> Self {
        let new_value = self.range.map(&range, self.value);
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
}
