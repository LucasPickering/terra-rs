use anyhow::anyhow;
use std::ops;

/// A macro to measure the evaluation time of an expression. Wraps an
/// expression, and outputs a tuple of the value of the expression with the
/// elapsed time, as a [Duration](std::time::Duration).
#[macro_export]
macro_rules! timed {
    ($ex:expr) => {{
        // use std::time::Instant;
        use std::time::Duration;

        // let start = Instant::now();
        let value = $ex;
        // let elapsed = Instant::now() - start;
        let elapsed = Duration::new(0, 0);
        (value, elapsed)
    }};
}

/// An RGB color. Values are stored as floats between 0 and 1 (inclusive).
/// This uses f32 because the extra precision from f64 is pointless.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color3 {
    red: f32,
    green: f32,
    blue: f32,
}

impl Color3 {
    /// Create a new RGB color. Return if any of the components are out of
    /// the range [0.0, 1.0].
    pub fn new(red: f32, green: f32, blue: f32) -> anyhow::Result<Self> {
        fn check_component(
            component_name: &str,
            value: f32,
        ) -> anyhow::Result<f32> {
            if (0.0..=1.0).contains(&value) {
                Ok(value)
            } else {
                Err(anyhow!(
                    "Color component {} must be in [0.0, 1.0], but was {}",
                    component_name,
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

    pub fn red(&self) -> f32 {
        self.red
    }

    pub fn green(&self) -> f32 {
        self.green
    }

    pub fn blue(&self) -> f32 {
        self.blue
    }
}

// Scale a color by a constant
impl ops::Mul<f32> for Color3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let range = <NumRange<f32>>::NORMAL;
        let red = range.clamp(self.red * rhs);
        let green = range.clamp(self.green * rhs);
        let blue = range.clamp(self.blue * rhs);
        Self::new(red, green, blue).unwrap()
    }
}

/// Any time that can be turned into a numeric range
pub trait Rangeable = Copy
    + Sized
    + ops::Add<Self, Output = Self>
    + ops::Sub<Self, Output = Self>
    + ops::Mul<Self, Output = Self>
    + ops::Div<Self, Output = Self>
    + PartialOrd;

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

    pub fn span(&self) -> T {
        self.max - self.min
    }

    /// Map a value from this range to the target range.
    pub fn map_to(&self, dest_range: &Self, value: T) -> T {
        let normalized = (value - self.min) / self.span();
        dest_range.min + (normalized * dest_range.span())
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

impl NumRange<f32> {
    /// The range [0, 1]
    pub const NORMAL: Self = Self::new(0.0, 1.0);

    /// Map a value from this range to the range [0, 1]
    pub fn normalize(&self, value: f32) -> f32 {
        self.map_to(&Self::NORMAL, value)
    }
}

impl NumRange<f64> {
    /// The range [0, 1]
    pub const NORMAL: Self = Self::new(0.0, 1.0);

    /// Map a value from this range to the range [0, 1]
    pub fn normalize(&self, value: f64) -> f64 {
        self.map_to(&Self::NORMAL, value)
    }
}
