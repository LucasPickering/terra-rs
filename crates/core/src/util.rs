use anyhow::anyhow;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub,
    SubAssign, Sum,
};
use log::debug;
use rand::{
    distributions::uniform::{SampleRange, SampleUniform, UniformSampler},
    RngCore,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A macro to unwrap an option to its `Some` value, and bail out of the current
/// function with an [anyhow::Error] if not. Can only be used in functions that
/// return an [anyhow::Result].
#[macro_export]
macro_rules! unwrap_or_bail {
    // ($msg:literal $(,)?) => { ... };
    // ($err:expr $(,)?) => { ... };
    ($opt:expr, $fmt:expr, $($arg:tt)*) => {
        match $opt {
            Some(v) => v,
            // None => bail!($fmt, $($arg)*)
            None => return Err(anyhow::anyhow!($fmt, $($arg)*)),
        }
    };
}

/// A macro to measure the evaluation time of an expression. Wraps an
/// expression, and outputs a tuple of the value of the expression with the
/// elapsed time, as a [Duration](std::time::Duration).
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! timed {
    ($label:expr, $ex:expr) => {
        timed!($label, log::Level::Debug, $ex)
    };
    ($label:expr, $log_level:expr, $ex:expr) => {{
        let now = std::time::Instant::now();
        let value = $ex;
        let elapsed = now.elapsed();
        log::log!($log_level, "{} took {} ms", $label, elapsed.as_millis());
        value
    }};
}

/// Re-implementation of the above macro for wasm
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
#[macro_export]
macro_rules! timed {
    ($label:expr, $ex:expr) => {
        // log level does nothing on the wasm version
        timed!($label, log::Level::Debug, $ex)
    };
    ($label:expr, $log_level:expr, $ex:expr) => {{
        use web_sys::console;

        // https://developer.mozilla.org/en-US/docs/Web/API/console/time
        console::time_with_label($label);
        let value = $ex;
        console::time_end_with_label($label);
        value
    }};
}

/// Unit used for elevation
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Display,
    PartialEq,
    PartialOrd,
    From,
    Into,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Sum,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{} m", "self.0")]
pub struct Meter(pub f64);

/// Unit used for tile area. One tile has a top surface area of 1m^2.
///
/// **Note:** Tiles may not actually be rendered such that the area is exactly
/// 1m^2 with reference to the elevation, but that's fine. This is just a nice
/// simplification that makes math easier.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Display,
    PartialEq,
    PartialOrd,
    From,
    Into,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Sum,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{} m²", "self.0")]
pub struct Meter2(pub f64);

/// Unit used for liquid volume. One meter of runoff on a single tile equals
/// 1 volumetric meter. See caveat on [Meter2], this may not actually appear
/// to be 1m*1m*1m when compared to elevation, depending on what ratios the
/// renderer uses.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Display,
    PartialEq,
    PartialOrd,
    From,
    Into,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Sum,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{} m³", "self.0")]
pub struct Meter3(pub f64);

// 1m * 1m^2 = 1m^3
impl ops::Mul<Meter> for Meter2 {
    type Output = Meter3;

    fn mul(self, rhs: Meter) -> Self::Output {
        Meter3(self.0 * rhs.0)
    }
}
impl ops::Mul<Meter2> for Meter {
    type Output = Meter3;

    fn mul(self, rhs: Meter2) -> Self::Output {
        Meter3(self.0 * rhs.0)
    }
}

// 1m^3 / 1m^2 = 1m
impl ops::Div<Meter2> for Meter3 {
    type Output = Meter;

    fn div(self, rhs: Meter2) -> Self::Output {
        Meter(self.0 / rhs.0)
    }
}

/// An RGB color. Values are stored as floats between 0 and 1 (inclusive).
/// This uses f32 because the extra precision from f64 is pointless.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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

    /// Create a new RGB color from integer components in the [0,255] range.
    pub fn new_int(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
        }
    }

    /// Convert this number to a set of 3 bytes: `(red, green, blue)`
    pub fn to_ints(self) -> (u8, u8, u8) {
        (
            (self.red * 255.0) as u8,
            (self.green * 255.0) as u8,
            (self.blue * 255.0) as u8,
        )
    }

    /// Convert this color to an HTML color code: `#rrggbb`
    pub fn to_html(self) -> String {
        let (r, g, b) = self.to_ints();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

// Scale a color by a constant
impl ops::Mul<f32> for Color3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let red = Self::COMPONENT_RANGE.clamp(self.red * rhs);
        let green = Self::COMPONENT_RANGE.clamp(self.green * rhs);
        let blue = Self::COMPONENT_RANGE.clamp(self.blue * rhs);
        // It's safe to bypass the constructor here because we just clamped
        // all 3 components to the valid range
        Self { red, green, blue }
    }
}

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

impl Rangeable<f64> for Meter3 {
    fn zero() -> Self {
        0.0.into()
    }

    fn one() -> Self {
        1.0.into()
    }
}

/// A range between two numeric values, inclusive on both ends.
#[derive(Copy, Clone, Debug, Display)]
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

    /// Max minus min
    pub fn span(&self) -> T {
        self.max - self.min
    }

    /// Create a [RangeValue] in this range, which can be more convenient for
    /// chaining operations.
    pub fn value(self, value: T) -> RangeValue<T, I> {
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

    /// Checks if the value is in this range. If it isn't, return an error.
    pub fn ensure_contains(&self, value: T) -> anyhow::Result<()> {
        if self.contains(value) {
            Ok(())
        } else {
            Err(anyhow!("value {} is not in range {}", value, self))
        }
    }

    /// Map a value from this range to the target range.
    pub fn map_to(&self, dest_range: &Self, value: T) -> T {
        let normalized = (value - self.min) / self.span().into();
        dest_range.min + (normalized * dest_range.span().into())
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

impl<T: Into<I> + Rangeable<I>, I> RangeValue<T, I> {
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

/// Compare two `PartialOrd` values dangerously. If the partial comparison
/// fails (returns `None`), this will panic. This is useful if you have floats
/// that you know for a fact will not be `NaN`.
pub fn cmp_unwrap<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    a.partial_cmp(b).unwrap()
}

/// Calculate the length of a world (the number of tiles it contains) based on
/// its radius. Radius 0 means 1 tile, 1 is 7 tiles, 2 is 19, etc.
pub fn world_len(radius: u16) -> usize {
    // We'll always have 3r^2+3r+1 tiles (a reduction of a geometric sum).
    // f(0) = 1, and we add 6r tiles for every step after that, so:
    // 1, (+6) 7, (+12) 19, (+18) 37, ...
    let r = radius as usize;
    3 * r * r + 3 * r + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_len() {
        assert_eq!(world_len(0), 1);
        assert_eq!(world_len(1), 7);
        assert_eq!(world_len(2), 19);
        assert_eq!(world_len(3), 37);
    }
}
