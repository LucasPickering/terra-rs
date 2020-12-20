use anyhow::anyhow;
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

    /// Map a value from this range to the target range.
    pub fn map(&self, dest_range: &Self, value: T) -> T {
        let normalized = (value - self.min) / self.span();
        dest_range.min + (normalized * dest_range.span())
    }

    /// Map a value from this range to the range [0, 1]
    pub fn normalize(&self, value: T) -> T {
        let normal_range = Self::new(T::zero(), T::one());
        self.map(&normal_range, value)
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

impl<T: Rangeable + Display> Display for NumRange<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}
