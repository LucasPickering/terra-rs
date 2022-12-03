use crate::NumRange;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Neg,
    Sub, SubAssign, Sum,
};
use serde::{Deserialize, Serialize};
use std::ops;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A 2D point in screen space. See module-level docs in [crate::hex] for a
/// description of what screen space means.
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
    Neg,
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
#[display(fmt = "({}, {})", "self.x", "self.y")]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl From<nalgebra::Point2<f64>> for Point2 {
    fn from(other: nalgebra::Point2<f64>) -> Self {
        Self {
            x: other.x,
            y: other.y,
        }
    }
}

/// An RGB color. Values are stored as floats between 0 and 1 (inclusive).
/// This uses f32 because the extra precision from f64 is pointless.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color3 {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color3 {
    /// The valid range of values for each component in RGB
    const COMPONENT_RANGE: NumRange<f32> = NumRange::new(0.0, 1.0);

    /// Create a new RGB color with components in the range [0.0, 1.0]. Panic
    /// if any of the components are out of range
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        fn check_component(component_name: &str, value: f32) -> f32 {
            if Color3::COMPONENT_RANGE.contains(value) {
                value
            } else {
                panic!(
                    "Color component {} must be in {}, but was {}",
                    component_name,
                    Color3::COMPONENT_RANGE,
                    value
                )
            }
        }

        Self {
            red: check_component("red", red),
            green: check_component("green", green),
            blue: check_component("blue", blue),
        }
    }

    /// Create a new RGB color from integer components in the [0,255] range.
    pub const fn new_int(red: u8, green: u8, blue: u8) -> Self {
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
        format!("#{r:02x}{g:02x}{b:02x}")
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
