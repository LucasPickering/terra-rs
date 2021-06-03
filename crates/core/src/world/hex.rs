//! This module holds basic types and data structures related to hexagon grids.

use derive_more::{Add, AddAssign, Display, Mul, MulAssign};
use fnv::FnvBuildHasher;
use indexmap::{map::Entry, IndexMap};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    hash::Hash,
    ops,
};
use strum::{EnumIter, IntoEnumIterator};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{
    util::{Point2, Vector2},
    Tile,
};

/// A point in a hexagon-tiled world. Each point has an x, y, and z component.
///
/// ## Hex Coordinate System
///
/// See this page for info on how the cube coordinate system works:
/// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
///
/// **In this page's vernacular, we use "flat topped" tiles.**
///
/// ## Implementation
///
/// This struct actually only needs to store x and y, since x+y+z=0 for all
/// points, so z can be derived as necessary which means we can save 33% of
/// the memory.
///
/// The x and y coordinates are stored as `i16`s. We'll never have a world with
/// a radius of more than 32k (that'd be ~4 billion tiles), so this saves on
/// memory a lot.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Serialize, Deserialize,
)]
#[display(fmt = "({}, {}, {})", "self.x()", "self.y()", "self.z()")]
pub struct HexPoint {
    x: i16,
    y: i16,
}

// Wasm functions
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HexPoint {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn x(&self) -> i16 {
        self.x
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn y(&self) -> i16 {
        self.y
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn z(&self) -> i16 {
        -(self.x + self.y)
    }

    /// Calculate the path distance between two points, meaning the number of
    /// hops it takes to get from one to the other. 0 if the points are equal,
    /// 1 if they are adjacent, 2 if there is 1 point between them, etc.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn distance_to(&self, other: HexPoint) -> usize {
        // https://www.redblobgames.com/grids/hexagons/#distances
        ((self.x() - other.x()).abs()
            + (self.y() - other.y()).abs()
            + (self.z() - other.z()).abs()) as usize
            / 2
    }

    /// Convert this position from hex space to 2D space. Useful for rendering
    /// a tile in 2D or 3D.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn to_point2(self) -> Point2 {
        Point2 {
            x: self.x as f64 * 1.5,
            y: (self.x as f64 / 2.0 + self.y as f64) * -(3.0f64.sqrt()),
        }
    }
}

// Non-wasm functions
impl HexPoint {
    pub const ORIGIN: Self = Self::new(0, 0);

    /// Alias for [Self::new_xy]
    pub const fn new(x: i16, y: i16) -> Self {
        Self::new_xy(x, y)
    }

    /// Construct a new hex point with the given x and y. Since x+y+z=0 for all
    /// points, we can derive z from x & y.
    pub const fn new_xy(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    /// Construct a new hex point with the given x and z. Since x+y+z=0 for all
    /// points, we can derive y from x & z.
    pub const fn new_xz(x: i16, z: i16) -> Self {
        Self::new(x, -x - z)
    }

    /// Construct a new hex point with the given y and z. Since x+y+z=0 for all
    /// points, we can derive x from y & z.
    pub const fn new_yz(y: i16, z: i16) -> Self {
        Self::new(-y - z, y)
    }

    /// Get an iterator of all the points directly adjacent to this one. The
    /// iterator will always contain exactly 6 values.
    pub fn adjacents(self) -> impl Iterator<Item = HexPoint> {
        HexDirection::iter().map(move |dir| self + dir.to_vector())
    }
}

impl ops::Add<HexVector> for HexPoint {
    type Output = HexPoint;

    fn add(self, rhs: HexVector) -> Self::Output {
        Self::new(self.x + rhs.x(), self.y + rhs.y())
    }
}

/// A vector in a hex world. This is an (x,y,z) kind of vector, not a list
/// vector. This is essentially the same as a [HexPoint], but by denoting some
/// values explicitly as vectors rather than points, it makes a bit clearer when
/// shifting points around. Like [HexPoint], x+y+z will always equal 0 for all
/// vectors.
#[derive(Copy, Clone, Debug, Display, Add, Mul, AddAssign, MulAssign)]
#[display(fmt = "({}, {}, {})", "self.x()", "self.y()", "self.z()")]
pub struct HexVector {
    x: i16,
    y: i16,
}

impl HexVector {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> i16 {
        self.x
    }

    pub fn y(&self) -> i16 {
        self.y
    }

    pub fn z(&self) -> i16 {
        -(self.x + self.y)
    }
}

/// A trait that denotes any type that has a singular assigned position in the
/// hex world.
pub trait HasHexPosition: Sized {
    fn position(&self) -> HexPoint;
}

/// The 6 directions in which hexes can line up side-to-side. This is similar to
/// [HexAxis], but while `HexAxis` is center-to-vertex, this enum denotes
/// center-to-side directions. So each entry in this enum reprensents a line
/// drawn from the center of a tile to the center of one side on that tile.
///
/// See this page for more info (we use "flat topped" tiles):
/// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
#[derive(
    Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum HexDirection {
    Up,
    UpRight,
    DownRight,
    Down,
    DownLeft,
    UpLeft,
}

impl HexDirection {
    /// Get the direction that is directly opposite this one
    pub fn opposite(self) -> HexDirection {
        match self {
            Self::Up => Self::Down,
            Self::UpRight => Self::DownLeft,
            Self::DownRight => Self::UpLeft,
            Self::Down => Self::Up,
            Self::DownLeft => Self::UpRight,
            Self::UpLeft => Self::DownRight,
        }
    }

    /// Get a pair of [HexAxialDirection]s that denote the endpoints of a side
    /// of a tile. The direction defines which side we're talking about, then
    /// the directions define the vertices relative to the center of the tile.
    /// These will always been returned in clockwise order.
    pub fn endpoints(self) -> (HexAxialDirection, HexAxialDirection) {
        match self {
            Self::Up => (HexAxialDirection::Y_POS, HexAxialDirection::Z_NEG),
            Self::UpRight => {
                (HexAxialDirection::Z_NEG, HexAxialDirection::X_POS)
            }
            Self::DownRight => {
                (HexAxialDirection::X_POS, HexAxialDirection::Y_NEG)
            }
            Self::Down => (HexAxialDirection::Y_NEG, HexAxialDirection::Z_POS),
            Self::DownLeft => {
                (HexAxialDirection::Z_POS, HexAxialDirection::X_NEG)
            }
            Self::UpLeft => {
                (HexAxialDirection::X_NEG, HexAxialDirection::Y_POS)
            }
        }
    }

    /// Get an vector offset that would move a point one tile in this direction
    pub fn to_vector(self) -> HexVector {
        match self {
            Self::Up => HexVector::new(0, 1),
            Self::UpRight => HexVector::new(1, 0),
            Self::DownRight => HexVector::new(1, -1),
            Self::Down => HexVector::new(0, -1),
            Self::DownLeft => HexVector::new(-1, 0),
            Self::UpLeft => HexVector::new(-1, 1),
        }
    }

    /// Convert this direction into a half-unit 2D offset. This offset
    /// represents the distance between the center of a tile and the midpoint of
    /// one side of the tile, in **2D** coordinates. See [Point2] for a
    /// description of 2D coordinates. This is probably only useful for
    /// rendering.
    ///
    /// https://www.redblobgames.com/grids/hexagons/#hex-to-pixel (FLAT TOPPED)
    pub fn to_vector2(self) -> Vector2 {
        match self {
            Self::Up => Vector2 {
                x: 0.0,
                y: -Tile::SIDE_RADIUS,
            },
            Self::UpRight => Vector2 {
                x: Tile::VERTEX_RADIUS * 0.75,
                y: -Tile::SIDE_RADIUS / 2.0,
            },
            Self::DownRight => Vector2 {
                x: Tile::VERTEX_RADIUS * 0.75,
                y: Tile::SIDE_RADIUS / 2.0,
            },
            Self::Down => Vector2 {
                x: 0.0,
                y: Tile::SIDE_RADIUS,
            },
            Self::DownLeft => Vector2 {
                x: -Tile::VERTEX_RADIUS * 0.75,
                y: Tile::SIDE_RADIUS / 2.0,
            },
            Self::UpLeft => Vector2 {
                x: -Tile::VERTEX_RADIUS * 0.75,
                y: -Tile::SIDE_RADIUS / 2.0,
            },
        }
    }
}

/// The 3 axes in our coordinate system.
///
/// See this page for more info (we use "flat topped" tiles):
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
/// See this page for more info (we use "flat topped" tiles):
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
    /// is due left of the origin), then goes around clockwise from there.
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

    /// Convert this axial direction into a half-unit 2D offset. This offset
    /// represents the distance between the center of a tile and one vertex,
    /// in **2D** coordinates. See [Point2] for a description of 2D coordinates.
    /// This is probably only useful for rendering.
    ///
    /// https://www.redblobgames.com/grids/hexagons/#hex-to-pixel (FLAT TOPPED)
    pub fn to_vector2(self) -> Vector2 {
        let vec2 = match self.axis {
            HexAxis::X => Vector2 {
                x: Tile::VERTEX_RADIUS,
                y: 0.0,
            },
            HexAxis::Y => Vector2 {
                x: -Tile::VERTEX_RADIUS / 2.0,
                y: -Tile::SIDE_RADIUS,
            },
            HexAxis::Z => Vector2 {
                x: -Tile::VERTEX_RADIUS / 2.0,
                y: Tile::SIDE_RADIUS,
            },
        };
        vec2 * (self.signum() as f64)
    }
}

/// A set of hex points
pub type HexPointSet = HashSet<HexPoint, FnvBuildHasher>;
/// A map of hex points to some `T`
pub type HexPointMap<T> = HashMap<HexPoint, T, FnvBuildHasher>;
/// An ORDERED map of hex points to some `T`. This has some extra memory
/// overhead, so we should only use it when we actually need the ordering.
pub type HexPointIndexMap<T> = IndexMap<HexPoint, T, FnvBuildHasher>;
/// A map of hex directions to some `T`
pub type HexDirectionMap<T> = HashMap<HexDirection, T, FnvBuildHasher>;

/// A static mapping of hex directions to values. This is similar to a
/// `HexDirectionMap<T>`, except that it always holds exactly 6 values and they
/// can be accessed via static fields. This is more useful in some cases,
/// especially post-world generation because we won't have to add or remove
/// values at that point. Having static fields makes serialization in external
/// apps a bit easier.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HexDirectionValues<T: Copy + Clone + Debug + PartialEq + Serialize> {
    pub up: T,
    pub up_right: T,
    pub down_right: T,
    pub down: T,
    pub down_left: T,
    pub up_left: T,
}

impl<T: Copy + Clone + Debug + PartialEq + Serialize> HexDirectionValues<T> {
    /// Copy all values in this struct into an array. The ordering will be the
    /// same as the iteration order of [HexDirection]: Clockwise, starting with
    /// `up`.
    pub fn as_array(&self) -> [T; 6] {
        [
            self.up,
            self.up_right,
            self.down_right,
            self.down,
            self.down_left,
            self.up_left,
        ]
    }
}

// Convert a dynamic map into a struct with fixed fields. Missing values will
// be populated with defaults
impl<T: Copy + Clone + Debug + Default + PartialEq + Serialize>
    From<HexDirectionMap<T>> for HexDirectionValues<T>
{
    fn from(mut map: HexDirectionMap<T>) -> Self {
        let mut helper =
            |dir: HexDirection| map.remove(&dir).unwrap_or_default();

        Self {
            up: helper(HexDirection::Up),
            up_right: helper(HexDirection::UpRight),
            down_right: helper(HexDirection::DownRight),
            down: helper(HexDirection::Down),
            down_left: helper(HexDirection::DownLeft),
            up_left: helper(HexDirection::UpLeft),
        }
    }
}

/// A cluster is a set of contiguous hex points. All items in a cluster are
/// adjacent to at least one other item in the cluster (unless the cluster is a
/// singular item).
#[derive(Clone, Debug)]
pub struct Cluster<T> {
    tiles: HexPointIndexMap<T>,
    adjacents: HexPointSet,
}

impl<T: Debug> Cluster<T> {
    /// Locate clusters of points within a map of tiles according to a
    /// predicate. All items that satisfy the predicate will be clustered
    /// such that any two satisfactory tiles that are adjacent to each other
    /// will be in a cluster together. The returned clusters hold mutables
    /// references to the items in this map, so they can be modified after
    /// clustering.
    ///
    /// The predicate returns a result, to allow for fallible operations during
    /// the check. If any predicate returns an error, the function will abort
    /// and return an error.
    pub fn predicate<P: Fn(&T) -> bool>(
        tiles: &mut HexPointMap<T>,
        predicate: P,
    ) -> Vec<Cluster<&'_ mut T>> {
        // Here's our algorithm:
        // - Create a pool of items that have yet to be clustered
        // - Grab a random item from the pool
        // - If it matches the predicate, do a BFS out from that item, including
        //   all items that match the predicate
        // - Once we run out of matchings items, consider the cluster complete
        // - Repeat with the remaining unclustered items

        // Copy our map into one that will hold the remaining items left to
        // check
        let mut remaining: HexPointIndexMap<&mut T> =
            tiles.iter_mut().map(|(pos, t)| (*pos, t)).collect();
        let mut clusters: Vec<Cluster<&mut T>> = Vec::new();

        // Grab the first unchecked item and start building a cluster around it.
        // This loop runs once per generated cluster, plus once per each failed
        // attempt at a cluster (where the first item fails the predicate)
        while let Some(first_entry) = remaining.pop() {
            let mut cluster = HexPointIndexMap::default();
            // Start our BFS. We'll use a queue of the next items to check, and
            // seed it with our first item. It doesn't seem to matter if we
            // allocate this on each loop or do it outside, so it's probably
            // getting optimized to the same thing
            let mut bfs_queue: VecDeque<(HexPoint, &mut T)> = VecDeque::new();
            // Start with this tile - if it fails the predicate it'll be the
            // only one we check for this cluster
            bfs_queue.push_back(first_entry);

            // Grab the next item off the queue and check it
            while let Some((pos, item)) = bfs_queue.pop_front() {
                if predicate(&item) {
                    // If it passes the pred, then add it to the cluster and add
                    // its neighbors to the queue
                    cluster.insert(pos, item);

                    // Remove all the adjacent items from the unclustered list
                    // map and add them to the queue
                    let rem = &mut remaining;
                    bfs_queue.extend(
                        pos.adjacents().filter_map(move |adj_pos| {
                            rem.remove_entry(&adj_pos)
                        }),
                    );
                }
            }

            if !cluster.is_empty() {
                clusters.push(Cluster::new(cluster));
            }
        }

        clusters
    }

    pub fn new(tiles: HexPointIndexMap<T>) -> Self {
        // Initialize the set of all tiles that are adjacent to (but not in) the
        // cluster
        let mut adjacents = HexPointSet::default();
        for pos in tiles.keys() {
            for adj in pos.adjacents() {
                if !tiles.contains_key(&adj) {
                    adjacents.insert(adj);
                }
            }
        }
        Self { tiles, adjacents }
    }

    /// A reference to the map of tiles in this cluster
    pub fn tiles(&self) -> &HexPointIndexMap<T> {
        &self.tiles
    }

    /// Move the tile map out of this struct
    pub fn into_tiles(self) -> HexPointIndexMap<T> {
        self.tiles
    }

    /// The set of positions that are directly adjacent to at least one tile in
    /// this cluster, but NOT in the cluster themselves. **These positions do
    /// not necessarily exist in the world!** The cluster has no context of the
    /// bounds of the world, so there's no guarantees around the validity of
    /// these neighbors.
    pub fn adjacents(&self) -> &HexPointSet {
        &self.adjacents
    }

    /// Add a new tile to the cluster. Returns an error if there is already a
    /// tile at that position, or if the new tile isn't contiguous to the
    /// cluster.
    pub fn insert(&mut self, pos: HexPoint, tile: T) {
        // Any tile we add in should already be a neighbor of the cluster. If
        // it isn't that means it's discontiguous which breaks the cardinal rule
        let removed = self.adjacents.remove(&pos);
        assert!(
            removed,
            "cannot add tile at {} to cluster {:?}, it is not adjacent!",
            pos, self
        );

        // Add the tile to the map, and add its neighbors to our set of
        // neighbors
        match self.tiles.entry(pos) {
            Entry::Vacant(entry) => {
                entry.insert(tile);
            }
            Entry::Occupied(_) => {
                panic!("tile {} is already in cluster {:?}", pos, self);
            }
        }
        let tiles = &self.tiles; // cause closure capturing is kinda dumb
        let new_neighbors = pos
            .adjacents()
            .filter(|adj_pos| !tiles.contains_key(adj_pos));
        self.adjacents.extend(new_neighbors);
    }

    /// Join this cluster with the other one. This assumes that the two clusters
    /// are already adjacent to each other, so that the resulting cluster
    /// remains contiguous. This assumption is not checked though! So be careful
    /// here.
    pub fn join(&mut self, other: Cluster<T>) {
        self.tiles.extend(other.tiles);
        // There may be a more efficient way to do this, but the naive method
        // is very easy to let's do that. We can optimize this if it ends up
        // being a bottleneck
        self.adjacents = self
            .adjacents
            .iter()
            .copied()
            .chain(other.adjacents.into_iter())
            .filter(|pos| !self.tiles.contains_key(pos))
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_to() {
        let p0 = HexPoint::ORIGIN;
        let p1 = HexPoint::new(-1, 1);
        let p2 = HexPoint::new(2, -1);
        let p3 = HexPoint::new(2, -3);

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
    fn test_hex_direction_map_to_values() {
        // Make sure each direction is mapped correctly
        let mut map: HexDirectionMap<usize> = HexDirectionMap::default();
        map.insert(HexDirection::Up, 1);
        map.insert(HexDirection::UpRight, 2);
        map.insert(HexDirection::DownRight, 3);
        map.insert(HexDirection::Down, 4);
        map.insert(HexDirection::DownLeft, 5);
        map.insert(HexDirection::UpLeft, 6);
        assert_eq!(
            HexDirectionValues::from(map),
            HexDirectionValues {
                up: 1,
                up_right: 2,
                down_right: 3,
                down: 4,
                down_left: 5,
                up_left: 6,
            }
        );

        // Should populate missing fields with the default
        assert_eq!(
            HexDirectionValues::from(HexDirectionMap::default()),
            HexDirectionValues {
                up: 0,
                up_right: 0,
                down_right: 0,
                down: 0,
                down_left: 0,
                up_left: 0,
            }
        );
    }
}
