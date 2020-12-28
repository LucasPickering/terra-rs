use derive_more::{
    Add, AddAssign, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign,
    Sum,
};
use wasm_bindgen::prelude::*;

/// Unit used for elevation
#[wasm_bindgen]
#[derive(
    Copy,
    Clone,
    Debug,
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
)]
pub struct Meter(pub f64);

/// Unit used for liquid volume. One meter of runoff on a single tile equals
/// 1 volumetric meter.
///
/// Note: this isn't *actually* equal to 1m*1m*1m, because the area of a tile
/// isn't 1m^2. But since we only deal in tile-sized increments of areas, this
/// is a convenient simplication.
#[wasm_bindgen]
#[derive(
    Copy,
    Clone,
    Debug,
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
)]
pub struct Meter3(pub f64);
