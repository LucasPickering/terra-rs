use crate::NumRange;
use anyhow::anyhow;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Neg,
    Sub, SubAssign, Sum,
};
use std::ops;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A point in 2D rendered space. This isn't used at all during world
/// generation/processing, but is useful during rendering. You can use
/// [HexPoint::to_point2] to convert a tile's world position into a renderable
/// 2D position. These positions aren't really useful outside of rendering, so
/// stick to [HexPoint] for stuff like distances, pathfinding, etc.
///
/// ## 2D Coordinates
///
/// Unlike hex coordinates, which have 3 components, 2D coordinates obviously
/// only have 2. This coordinate system uses the center of the screen as the
/// origin, so the tile with the hex position of `(0, 0, 0)` will be centered on
/// `(0, 0)` in 2D. Left is negative x, right is positive x. Down is positive y,
/// up is negative y.
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
)]
#[display(fmt = "({}, {})", x, y)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

/// A vector in 2D space. Like [Point2], this isn't used during world generation
/// at all, but is useful during rendering. This can represent offsets in 2D.
///
/// See [Point2] for a description of the 2D coordinate space.
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
)]
#[display(fmt = "({}, {})", x, y)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl ops::Add<Vector2> for Point2 {
    type Output = Point2;

    fn add(self, rhs: Vector2) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
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
