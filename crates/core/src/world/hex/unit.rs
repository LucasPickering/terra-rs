//! This sub-module contains basic types for units that form the hex coordinate
//! system. See the parent module documentation for more info on the coordinate
//! system.

use anyhow::anyhow;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    f64,
    fmt::{Debug, Display},
    hash::Hash,
    ops::Add,
};
use strum::{EnumIter, IntoEnumIterator};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// TODO make all From/TryFrom impls in here more generic after https://github.com/rust-lang/rust/issues/31844

/// A trait representing any three-component value in the hex coordinate system.
/// Any struct with an x/y/z that is part of the hex coordinate system should
/// implement this trait.
pub trait HexCoordinateValue: Sized {
    /// The primitive type of each component of the point. Must be convertible
    /// to `f64` so it can be mathematically converted to screen space
    type Component: Into<f64>;

    /// The `x` component of the coordinate
    fn x(&self) -> Self::Component;

    /// The `y` component of the coordinate
    fn y(&self) -> Self::Component;

    /// The `z` component of the coordinate
    fn z(&self) -> Self::Component;
}

/// A trait representing any point in the hex coordinate system. This is an
/// extension of [HexCoordinateValue]. By defining this as a trait, we can
/// define individual structs to represent different classes of points in the
/// system. E.g. some points may want to use integers internally, some may use,
/// floats, etc.
///
/// This trait provides whatever implementations it can so that all hex
/// points get common functionality for free.
///
/// See module-level docs for a description of the hex coordinate system.
pub trait HexThing: HexCoordinateValue {
    /// Construct a new point from the given components. This will validate
    /// the input, and if the given coordinates are invalid, will return an
    /// error. The "base" level of validation is that all points must
    /// fall on the 3D step function, but certain point types may provide
    /// further validation as necessary.
    fn new(
        x: Self::Component,
        y: Self::Component,
        z: Self::Component,
    ) -> anyhow::Result<Self>;
}

/// A point in the hex coordinate system that refers to a whole tile (via its
/// center point). This is a special case of [HexPoint] that always refers to
/// a tile center.
/// Applying this restriction has two benefits:
///
/// - Type safety for situations where we only want to refer to whole tiles
/// - Smaller and more specific component values with integers instead of floats
///
/// See module-level documentation for a description of the hex coordinate
/// system.
///
/// ## Implementation
///
/// By definition of our coordinate system, each tile center is defined as the
/// points of intersection between the 3D step function and the plane `x + y + z
/// = 0`. As such, this struct actually only needs to store `x` and `y` and can
/// derive `z` as needed. This means we can reduce our memory footprint by one
/// third.
///
/// The x and y coordinates are stored as `i16`s. We'll never have a world with
/// a radius of more than 32k (that'd be ~4 billion tiles), so this saves on
/// a lot of memory.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize,
)]
#[display(fmt = "({}, {}, {})", "self.x()", "self.y()", "self.z()")]
pub struct TilePoint {
    x: i16,
    y: i16,
}

impl TilePoint {
    pub const ORIGIN: Self = Self::new_xy(0, 0);

    /// Construct a new tile point with the given x and y. Since x+y+z=0 for all
    /// points, we can derive z from x & y.
    pub const fn new_xy(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    /// Construct a new tile point with the given x and z. Since x+y+z=0 for all
    /// points, we can derive y from x & z.
    pub const fn new_xz(x: i16, z: i16) -> Self {
        Self::new_xy(x, -x - z)
    }

    /// Construct a new tile point with the given y and z. Since x+y+z=0 for all
    /// points, we can derive x from y & z.
    pub const fn new_yz(y: i16, z: i16) -> Self {
        Self::new_xy(-y - z, y)
    }

    /// Get the location of a particular vertex of this tile
    pub fn vertex(self, direction: VertexDirection) -> TileVertexPoint {
        let vector = direction.to_vector();
        // Generic-ize this point so we can add it with the vector, then convert
        // it back to a tile vertex. This _shouldn't_ ever fail, unless
        // to_vector spits out a bad vector
        let point: HexPoint<i16> = self.into();
        let unchecked = point.translate(vector);
        unchecked.try_into().unwrap()
    }

    /// Get the midpoint of one side of this tile. The side is determined by the
    /// given direction.
    pub fn side_midpoint(self, direction: TileDirection) -> HexPoint<f64> {
        // To get midpoint, we average the two vertices on either side
        // We CAN'T just average the two tile centers (ours and the adjacents)
        // because that point won't fall on the step function
        let (left_dir, right_dir) = direction.adjacent_vertex_directions();
        let left: HexPoint<f64> = self.vertex(left_dir).into();
        let right: HexPoint<f64> = self.vertex(right_dir).into();
        let unchecked = left.translate(right) / 2.0;
        unchecked.try_into().unwrap()
    }

    /// Get the location of a particular tile adjacent to this one
    pub fn adjacent(self, direction: TileDirection) -> TilePoint {
        let vector = direction.to_vector();
        // Generic-ize this point so we can add it with the vector, then convert
        // it back to a tile point. This _shouldn't_ ever fail, unless to_vector
        // spits out a bad vector
        let point: HexPoint<i16> = self.into();
        point.translate(vector).try_into().unwrap()
    }

    /// Get an iterator of all the tile points directly adjacent to this one.
    /// The iterator will always contain exactly 6 values.
    pub fn adjacents(self) -> impl Iterator<Item = TilePoint> {
        TileDirection::iter().map(move |dir| self.adjacent(dir))
    }

    /// Calculate the path distance between two tiles, meaning the number of
    /// hops it takes to get from one to the other. 0 if the points are equal,
    /// 1 if the tiles are adjacent, 2 if there is 1 tile between them, etc.
    pub fn distance_to(self, other: TilePoint) -> usize {
        // https://www.redblobgames.com/grids/hexagons/#distances
        ((self.x() - other.x()).abs()
            + (self.y() - other.y()).abs()
            + (self.z() - other.z()).abs()) as usize
            // IMPORTANT: We divide by 2 here because two adjacent tile centers
            // are always separated by two cube edges
            / 2
    }
}

impl HexCoordinateValue for TilePoint {
    type Component = i16;

    fn x(&self) -> Self::Component {
        self.x
    }

    fn y(&self) -> Self::Component {
        self.y
    }

    fn z(&self) -> Self::Component {
        -(self.x + self.y)
    }
}

impl HexThing for TilePoint {
    fn new(x: i16, y: i16, z: i16) -> anyhow::Result<Self> {
        // We only need to check that this point falls on the plane x+y+z=0, we
        // don't need to explicitly check if it's on the step function. This is
        // because the intersection between the steps and the plane is _exactly_
        // the set of all integer points on the plane, so if we know we're on
        // the plane then the point must be valid
        if x + y + z != 0 {
            Err(anyhow!(
                "Invalid tile point ({}, {}, {}); must be on the plane x+y+z=0",
                x,
                y,
                z
            ))
        } else {
            Ok(Self::new_xy(x, y))
        }
    }
}

impl TryFrom<UncheckedHexPoint<i16>> for TilePoint {
    type Error = anyhow::Error;

    fn try_from(value: UncheckedHexPoint<i16>) -> Result<Self, Self::Error> {
        Self::new(value.x, value.y, value.z)
    }
}

/// A point in the hex coordinate system that refers to one of a tile's 6
/// vertices. This is a special case of [HexPoint] that always refers to a tile
/// center. Applying this restriction has two benefits:
///
/// - Type safety for situations where we only want to refer only to tile
///   vertices
/// - Smaller and more specific component values with integers instead of floats
///
/// See module-level documentation for a description of the hex coordinate
/// system. It's rare to construct these values directly; most often, you can
/// just use [TilePoint::vertex] to get a particular vertex for a tile.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize,
)]
#[display(fmt = "({}, {}, {})", "self.x()", "self.y()", "self.z()")]
pub struct TileVertexPoint {
    x: i16,
    y: i16,
    z: i16,
}

impl HexCoordinateValue for TileVertexPoint {
    type Component = i16;

    fn x(&self) -> Self::Component {
        self.x
    }

    fn y(&self) -> Self::Component {
        self.y
    }

    fn z(&self) -> Self::Component {
        self.z
    }
}

impl HexThing for TileVertexPoint {
    fn new(x: i16, y: i16, z: i16) -> anyhow::Result<Self> {
        // TODO enforce that the vertex falls on the step function

        // If the point falls on the x+y+z=0 plane, that means it's a tile
        // center, not a vertex
        if x + y + z == 0 {
            Err(anyhow!("Invalid tile vertex coordinates for a tile vertex: ({}, {}, {})", x, y, z))
        } else {
            Ok(Self { x, y, z })
        }
    }
}

impl TryFrom<UncheckedHexPoint<i16>> for TileVertexPoint {
    type Error = anyhow::Error;

    fn try_from(value: UncheckedHexPoint<i16>) -> Result<Self, Self::Error> {
        Self::new(value.x, value.y, value.z)
    }
}

/// A point in the hex coordinate system. This type can represent _any_ valid
/// point in the system, including all points covered by other point types.
/// The component data type for this struct is generic, to allow for
/// compatibility with both integer and float points.
#[derive(Copy, Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {}, {})", "self.x", "self.y", "self.z")]
pub struct HexPoint<T: Copy + Display> {
    x: T,
    y: T,
    z: T,
}

impl<T: Copy + Display + Into<f64> + Add<T, Output = T>> HexPoint<T> {
    fn translate(
        self,
        translation: impl HexCoordinateValue<Component = T>,
    ) -> UncheckedHexPoint<T> {
        UncheckedHexPoint {
            x: self.x() + translation.x(),
            y: self.y() + translation.y(),
            z: self.z() + translation.z(),
        }
    }
}

impl<T: Copy + Display + Into<f64>> HexCoordinateValue for HexPoint<T> {
    type Component = T;

    fn x(&self) -> Self::Component {
        self.x
    }

    fn y(&self) -> Self::Component {
        self.y
    }

    fn z(&self) -> Self::Component {
        self.z
    }
}

impl<T: Copy + Display + Into<f64>> HexThing for HexPoint<T> {
    fn new(x: T, y: T, z: T) -> anyhow::Result<Self> {
        // TODO enforce that the vertex falls on the step function
        Ok(Self { x, y, z })
    }
}

impl From<TilePoint> for HexPoint<i16> {
    fn from(other: TilePoint) -> Self {
        // TilePoint is a subset of HexPoint, so this should never fail
        Self::new(other.x(), other.y(), other.z()).unwrap()
    }
}

impl<T: HexThing<Component = i16>> From<T> for HexPoint<f64> {
    fn from(other: T) -> Self {
        // All other points are a subset of HexPoint, so this should never fail
        Self::new(other.x().into(), other.y().into(), other.z().into()).unwrap()
    }
}

/// An unvalidated version of [HexPoint]. An instance of this struct could
/// represent a point that doesn't fall on the 3D step function, meaning it
/// doesn't represent any valid point in hex space. This type is useful for
/// intermediate values during operations, but shouldn't be exposed outside
/// this module. This has a`TryInto` impls to convert back to validated point
/// types.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Display,
    Add,
    Sub,
    Mul,
    Div,
    Serialize,
    Deserialize,
)]
#[display(fmt = "({}, {}, {})", "self.x", "self.y", "self.z")]
struct UncheckedHexPoint<T: Display> {
    x: T,
    y: T,
    z: T,
}

impl<T: Copy + Display + Into<f64>> TryFrom<UncheckedHexPoint<T>>
    for HexPoint<T>
{
    type Error = anyhow::Error;

    fn try_from(value: UncheckedHexPoint<T>) -> Result<Self, Self::Error> {
        HexPoint::new(value.x, value.y, value.z)
    }
}

/// A vector in a hex world. This is an `(x, y, z)` kind of vector, not a list
/// vector. A vector represents some positional translation within the hex
/// coordinate system.
///
/// ## Validation
///
/// Unlike points, hex vectors **cannot be validated.** This is because a vector
/// could apply a valid translation (where the output remains valid) to one
/// point but an invalid translation to another point. As such, any hex vector
/// can be created and any time a point is transformed, it should be
/// re-validated.
#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
)]
#[display(fmt = "({}, {}, {})", "self.x", "self.y", "self.z")]
pub struct HexVector<T: Copy + Display> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy + Display + Into<f64>> HexCoordinateValue for HexVector<T> {
    type Component = T;

    fn x(&self) -> Self::Component {
        self.x
    }

    fn y(&self) -> Self::Component {
        self.y
    }

    fn z(&self) -> Self::Component {
        self.z
    }
}

impl<T: Copy + Display + Into<f64>> HexVector<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

/// A trait that denotes any data type that has a singular assigned position in
/// the hex world.
pub trait HasHexPosition: Sized {
    type Point: HexThing;

    fn position(&self) -> Self::Point;
}

/// The 3 axes in our coordinate system.
///
/// See this page for more info (we use "pointy topped" tiles):
/// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum HexAxis {
    X,
    Y,
    Z,
}

/// Similar to [HexDirection], but instead of denoting center-to-side
/// directions, this denotes center-to-vertex axes. Each main axis is made by
/// connecting two opposite vertices on the origin tile (3 vertex pairs = 3
/// axes). This enum splits each axis into two segments (positive and negative)
/// for ease of use.
///
/// See this page for more info (we use "pointy topped" tiles):
/// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HexAxialDirection {
    pub axis: HexAxis,
    pub positive: bool,
}

impl HexAxialDirection {
    pub const X_NEG: Self = Self {
        axis: HexAxis::X,
        positive: false,
    };
    pub const X_POS: Self = Self {
        axis: HexAxis::X,
        positive: true,
    };
    pub const Y_NEG: Self = Self {
        axis: HexAxis::Y,
        positive: false,
    };
    pub const Y_POS: Self = Self {
        axis: HexAxis::Y,
        positive: true,
    };
    pub const Z_NEG: Self = Self {
        axis: HexAxis::Z,
        positive: false,
    };
    pub const Z_POS: Self = Self {
        axis: HexAxis::Z,
        positive: true,
    };

    /// A list of all hex axial directions. Starts with the negative x (which
    /// is southwest of the origin), then goes around clockwise from there.
    pub const ALL: &'static [Self] = &[
        Self::X_NEG,
        Self::X_POS,
        Self::Y_NEG,
        Self::Y_POS,
        Self::Z_NEG,
        Self::Z_POS,
    ];
    /// List of all hex axial directions, in clockwise order starting from the
    /// top-left vertex of a tile.
    pub const CLOCKWISE: &'static [Self] = &[
        Self::Y_POS,
        Self::Z_NEG,
        Self::X_POS,
        Self::Y_NEG,
        Self::Z_POS,
        Self::X_NEG,
    ];

    pub fn signum(self) -> i16 {
        if self.positive {
            1
        } else {
            -1
        }
    }
}

/// A linear direction in a hex world. Similar to how the [HexPoint] trait
/// represents position, this trait represents direction. There are multiple
/// classes of direction though, that are often used mutually exclusively, so
/// this trait allows for multiple struct implementations that can be used
/// independently of each other.
///
/// Directions are defined using the three tiers of the cardinal direction
/// system. For any oredering, they will start at `north` and proceed clockwise
/// from there. Here is the full list of possible directions (in that order):
///
/// North, north-northeast, northeast, east-northeast, east, east-southeast,
/// southeast, south-southeast, south, south-southwest, southwest,
/// west-southwest, west, west-northwest, northwest, north-northwest
///
/// Each implementation of this trait may opt to include some of all of these
/// directions, depending on what sorts of headings it intends to represent.
/// For any given implementation, we call the possible directions that it allows
/// its  **"class"** of directions.
pub trait HexDirection: 'static + Copy + Eq + Sized {
    /// A list of all directions in an implementation's class, in clockwise
    /// order around the compass, starting at North. North may not necessarily
    /// be in the class, in which case the first element of this list will be
    /// whatever direction first follows North in the clockwise ordering.
    const CLOCKWISE: &'static [Self];

    /// Get the index of the given direction within the clockwise ordering of
    /// this class
    fn clockwise_index(self) -> usize {
        Self::CLOCKWISE.iter().position(|dir| self == *dir).unwrap()
    }

    /// Get the direction that is directly opposite this one. This will only
    /// account for directions _within_ this class, meaning that each direction
    /// in the class **must have its opposite present in the same class**.
    fn opposite(self) -> Self {
        let index = self.clockwise_index();
        let clockwise = Self::CLOCKWISE;
        let len = clockwise.len();
        // Because we assume the direction hsa an opposite in this class, we
        // can assume the number of directions is even
        clockwise[(index + (len / 2)) % len]
    }

    /// Convert this direction into a generic vector in the world coordinate
    /// system. Each component of the returned vector will be one of `0`, `1`,
    /// or `-1`. The exact purpose of the returned vector will vary for
    /// different implementations of this trait.
    fn to_vector(self) -> HexVector<i16>;
}

/// The 6 directions in which hexes can line up side-to-side. FOr any given
/// tile, a tile direction can represent two useful things:
///
/// - Direction from center point to the midpoint of a single side of that tile
/// - Direction to a neighboring tile's center point
#[derive(
    Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum TileDirection {
    /// North-northeast
    NNE,
    /// East
    E,
    /// South-southeast
    SSE,
    /// South-southwest
    SSW,
    /// West
    W,
    /// North-northwest
    NNW,
}

impl HexDirection for TileDirection {
    const CLOCKWISE: &'static [Self] =
        &[Self::NNE, Self::E, Self::SSE, Self::SSW, Self::W, Self::NNW];

    /// Get an vector offset that would move a point one tile in this direction
    fn to_vector(self) -> HexVector<i16> {
        match self {
            Self::NNE => HexVector::new(-1, 0, 1),
            Self::E => HexVector::new(-1, 1, 0),
            Self::SSE => HexVector::new(0, 1, -1),
            Self::SSW => HexVector::new(1, 0, -1),
            Self::W => HexVector::new(1, -1, 0),
            Self::NNW => HexVector::new(0, -1, 1),
        }
    }
}

impl TileDirection {
    /// Get the two [VertexDirection]s that are adjacent to this tile direction
    /// on the compass. I.e. if this direction points from the tile's center
    /// to the midpoint of a side, then the returned vertex directions will
    /// point from the center to either endpoint of that same side.
    pub fn adjacent_vertex_directions(
        self,
    ) -> (VertexDirection, VertexDirection) {
        let index = self.clockwise_index();
        let clockwise = VertexDirection::CLOCKWISE;
        let left = clockwise[index];
        let right = clockwise[(index + 1) % clockwise.len()];
        (left, right)
    }
}

/// The 6 directions you can go from the center of a tile to one of its
/// vertices. This is similar to [TileDirection], but while that struct is
/// center-to-center, this enum denotes center-to-vertex directions. The
/// coordinate system is constructed such that each of these directions lines
/// up visually with an axis. **But each direction here is not equivalent to a
/// single axial direction**. Some variants of this enum correspond to a single
/// step on a single axis, but others require multiple steps.
#[derive(
    Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum VertexDirection {
    /// North
    N,
    /// East-northeast
    ENE,
    /// East-southeast
    ESE,
    /// South
    S,
    /// West-southwest
    WSW,
    /// West-northwest
    WNW,
}

impl HexDirection for VertexDirection {
    const CLOCKWISE: &'static [Self] =
        &[Self::N, Self::ENE, Self::ESE, Self::S, Self::WSW, Self::WNW];

    /// Get an vector offset that would translate a point from the center
    /// of a tile to one of its vertices (in the corresponding direction).
    /// 3 of these offsets are a single step, while the others are two steps.
    /// In the cube model of the coordinate system, each tile's position is
    /// corresponds to the foremost vertex of the cube (as defined by the
    /// camera perspective). From that vertex, the 6 remaining visible vertices
    /// represent the outer vertices of the tile. Of those 6, 3 are only one
    /// edge away, while the other 3 are two edges. Consider this: any vertex
    /// on a cube is directly connected to exactly 3 other vertices. Therefore
    /// we can't reach all 6 with only a single integer step.
    fn to_vector(self) -> HexVector<i16> {
        match self {
            Self::N => HexVector::new(-1, -1, 0),
            Self::ENE => HexVector::new(-1, 0, 0),
            Self::ESE => HexVector::new(-1, 0, -1),
            Self::S => HexVector::new(0, 0, -1),
            Self::WSW => HexVector::new(0, -1, -1),
            Self::WNW => HexVector::new(0, -1, 0),
        }
    }
}

impl VertexDirection {
    /// Get the two [TileDirection]s that are adjacent to this vertex direction
    /// on the compass. I.e. if this direction points from the tile's center
    /// to one of its vertices, then the returned tile directions will point
    /// from the center to the midpoints of either side flanking that vertex.
    pub fn adjacent_tile_directions(self) -> (TileDirection, TileDirection) {
        let index = self.clockwise_index();
        let clockwise = TileDirection::CLOCKWISE;
        // Get the previous and next directions in the tile dir sequence. To
        // get prev, we just want clockwise[(index - 1) % len], but that has
        // two issues:
        // - % is remainder, *not* modulus, so `-1 % 6 == -1`, not 5
        // - We would have to convert from usize to isize to allow negatives
        // Those are both solvable, but the easier option is to just add 5
        // instead of subtracting 1, since those provide the same post-modulus
        // result.
        // ASSUMPTION: clockwise.len() is >0, so the -1 will never result in a
        // negative number
        let left = clockwise[(index + clockwise.len() - 1) % clockwise.len()];
        let right = clockwise[index];
        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_to() {
        let p0 = TilePoint::ORIGIN;
        let p1 = TilePoint::new_xy(-1, 1);
        let p2 = TilePoint::new_xy(2, -1);
        let p3 = TilePoint::new_xy(2, -3);

        assert_eq!(p0.distance_to(p0), 0);
        assert_eq!(p3.distance_to(p3), 0);

        assert_eq!(p0.distance_to(p1), 1);
        assert_eq!(p0.distance_to(p2), 2);
        assert_eq!(p0.distance_to(p3), 3);

        assert_eq!(p1.distance_to(p2), 3);
        assert_eq!(p1.distance_to(p3), 4);
        assert_eq!(p2.distance_to(p3), 2);
    }

    #[test]
    fn test_adjacent_vertex_directions() {
        assert_eq!(
            TileDirection::NNE.adjacent_vertex_directions(),
            (VertexDirection::N, VertexDirection::ENE)
        );
        assert_eq!(
            TileDirection::E.adjacent_vertex_directions(),
            (VertexDirection::ENE, VertexDirection::ESE)
        );
        assert_eq!(
            TileDirection::SSE.adjacent_vertex_directions(),
            (VertexDirection::ESE, VertexDirection::S)
        );
        assert_eq!(
            TileDirection::SSW.adjacent_vertex_directions(),
            (VertexDirection::S, VertexDirection::WSW)
        );
        assert_eq!(
            TileDirection::W.adjacent_vertex_directions(),
            (VertexDirection::WSW, VertexDirection::WNW)
        );
        assert_eq!(
            TileDirection::NNW.adjacent_vertex_directions(),
            (VertexDirection::WNW, VertexDirection::N)
        );
    }

    #[test]
    fn test_adjacent_tile_directions() {
        assert_eq!(
            VertexDirection::N.adjacent_tile_directions(),
            (TileDirection::NNW, TileDirection::NNE)
        );
        assert_eq!(
            VertexDirection::ENE.adjacent_tile_directions(),
            (TileDirection::NNE, TileDirection::E)
        );
        assert_eq!(
            VertexDirection::ESE.adjacent_tile_directions(),
            (TileDirection::E, TileDirection::SSE)
        );
        assert_eq!(
            VertexDirection::S.adjacent_tile_directions(),
            (TileDirection::SSE, TileDirection::SSW)
        );
        assert_eq!(
            VertexDirection::WSW.adjacent_tile_directions(),
            (TileDirection::SSW, TileDirection::W)
        );
        assert_eq!(
            VertexDirection::WNW.adjacent_tile_directions(),
            (TileDirection::W, TileDirection::NNW)
        );
    }
}
