use anyhow::anyhow;

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
        Ok(Self {
            red: Self::check_component("red", red)?,
            green: Self::check_component("green", green)?,
            blue: Self::check_component("blue", blue)?,
        })
    }

    /// Ensure that the given color component is between 0 and 1 (inclusive).
    /// If it is, return the given value. If not, return an error.
    fn check_component(
        component_name: &str,
        value: f32,
    ) -> anyhow::Result<f32> {
        if 0.0 <= value && value <= 1.0 {
            Ok(value)
        } else {
            Err(anyhow!(
                "Color component {} must be in [0.0, 1.0], but was {}",
                component_name,
                value
            ))
        }
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

/// A range between two float values, inclusive on both ends.
#[derive(Copy, Clone, Debug)]
pub struct FloatRange {
    pub min: f64,
    pub max: f64,
}

impl FloatRange {
    /// The range [0.0, 1.0].
    pub const NORMAL_RANGE: Self = Self::new(0.0, 1.0);

    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn span(&self) -> f64 {
        self.max - self.min
    }

    pub fn map_to(&self, dest_range: &Self, value: f64) -> f64 {
        let normalized = (value - self.min) / self.span();
        (normalized * dest_range.span()) + dest_range.min
    }
}
