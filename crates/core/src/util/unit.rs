use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub,
    SubAssign, Sum,
};
use serde::{Deserialize, Serialize};
use std::ops;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
