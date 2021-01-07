//! This module holds basic types and data structures related to hexagon grids.

use derive_more::{Add, AddAssign, Display, Mul, MulAssign};
use fnv::FnvBuildHasher;
use indexmap::IndexMap;
use std::{
    cmp,
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    hash::Hash,
    iter, ops,
};
use strum::{EnumIter, IntoEnumIterator};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A point in a hexagon-tiled world. Each point has an x, y, and z component.
/// See this page for info on how the cube coordinate system works:
/// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
///
/// **In this page's vernacular, we use "flat topped" tiles.**
///
/// This struct actually only needs to store x and y, since x+y+z=0 for all
/// points, so z can be derived as necessary which means we can save 33% of
/// the memory.
///
/// The x and y coordinates are stored as `i16`s. We'll never have a world with
/// a radius of more than 32k (that'd be ~4 billion tiles), so this saves on
/// memory a lot.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display)]
#[display(fmt = "({}, {}, {})", "self.x()", "self.y()", "self.z()")]
pub struct HexPoint {
    x: i16,
    y: i16,
}

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

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn distance_to(&self, other: HexPoint) -> usize {
        *[
            (self.x() - other.x()).abs(),
            (self.y() - other.y()).abs(),
            (self.z() - other.z()).abs(),
        ]
        .iter()
        .max()
        .unwrap() as usize
    }
}

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
        HexDirection::iter().map(move |dir| self + dir.vec())
    }
}

impl ops::Add<HexVec> for HexPoint {
    type Output = HexPoint;

    fn add(self, rhs: HexVec) -> Self::Output {
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
pub struct HexVec {
    x: i16,
    y: i16,
}

impl HexVec {
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

/// A map of tiles keyed by hex point, in a super hexagon pattern (the tiles
/// make up the shape of a larger hexagon). For a world of radius `r`, the
/// furthest tiles are all `r` steps from the center. This restriction is very
/// important because it allows us to optimize a lot by storing all the items in
/// a flat vector.
#[derive(Clone, Debug, Default)]
pub struct WorldMap<T> {
    /// Distance from the center of the world to the edge. 0 means the world is
    /// exactly 1 tile. 1 means 7 tiles, and so on. We frequently need to treat
    /// this as an `i16` as well, so this cannot exceed [std::isize::MAX]!
    /// If it does, we're gonna have bigger problems anyway...
    world_radius: u16,
    /// Each tile, keyed by its position.
    map: HexPointMap<T>,
}

impl<T> WorldMap<T> {
    /// Initialize a new world with the given radius.
    ///
    /// ## Arguments
    ///
    /// - `radius`: Distance from the origin to the edge of the world, in all
    ///   directions. 0 means a world of 1 tile, 1 is 7 tiles, 2 => 19, etc.
    /// - `initializer`: Function called to initialize each item in the world,
    ///   based on its position
    pub fn new(radius: u16, initializer: impl Fn(HexPoint) -> T) -> Self {
        let capacity = Self::world_size(radius);
        let mut map = HexPointMap::with_capacity_and_hasher(
            capacity,
            FnvBuildHasher::default(),
        );

        // Initialize a set of tiles with no data
        let r = radius as i16;
        for x in -r..=r {
            // If we just do [-r,r] for y as well, then we end up with a diamond
            // pattern instead of a super hexagon
            // https://www.redblobgames.com/grids/hexagons/#range
            let y_min = cmp::max(-r, -x - r);
            let y_max = cmp::min(r, -x + r);
            for y in y_min..=y_max {
                let pos = HexPoint::new(x, y);
                map.insert(pos, initializer(pos));
            }
        }
        debug_assert_eq!(map.len(), capacity, "expected 3r²+3r+1 tiles");

        Self {
            world_radius: radius,
            map,
        }
    }

    /// Calculate the size of a world (the number of tiles it contains) based on
    /// its radius. Radius 0 means 1 tile, 1 is 7 tiles, 2 is 19, etc.
    fn world_size(radius: u16) -> usize {
        // We'll always have 3r^2+3r+1 tiles (a reduction of a geometric sum).
        // f(0) = 1, and we add 6r tiles for every step after that, so:
        // 1, (+6) 7, (+12) 19, (+18) 37, ...
        let r = radius as usize;
        3 * r * r + 3 * r + 1
    }

    pub fn get(&self, pos: HexPoint) -> Option<&T> {
        self.map.get(&pos)
    }

    pub fn get_mut(&mut self, pos: HexPoint) -> Option<&mut T> {
        self.map.get_mut(&pos)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.map.iter().map(|(_, v)| v)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.map.iter_mut().map(|(_, v)| v)
    }

    /// Map this collection into a new collection by apply the given mapping
    /// function over each element. We need this instead of just relying on
    /// [std::iter::Iterator::map] because you can't collect an iterator into a
    /// [WorldMap].
    pub fn map<U>(self, f: impl Fn(T) -> U) -> WorldMap<U> {
        let map = self.map.into_iter().map(|(k, v)| (k, f(v))).collect();
        WorldMap {
            world_radius: self.world_radius,
            map,
        }
    }

    /// Get the number of items in the map
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl<T: Debug + HasHexPosition> WorldMap<T> {
    /// Locate clusters of points within this map according to a predicate. All
    /// items that satisfy the predicate will be clustered such that any two
    /// satisfactory tiles that are adjacent to each other will be in a cluster
    /// together. The returned cluster holds mutables references to the
    /// items in this map, so they can be modified after clustering.
    ///
    /// Potential optimization: we could definitely improve this by leveraging
    /// the full world map to do lookups faster. The lifetimes are a bit of
    /// a bitch but it would save a lot on hash lookups.
    pub fn clusters_predicate<P: Fn(&T) -> bool>(
        &mut self,
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
            self.iter_mut().map(|t| (t.position(), t)).collect();
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
}

impl<T> IntoIterator for WorldMap<T> {
    type Item = T;
    #[allow(clippy::type_complexity)]
    type IntoIter = iter::Map<
        <HexPointMap<T> as IntoIterator>::IntoIter,
        fn((HexPoint, T)) -> T,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter().map(|(_, v)| v)
    }
}

/// A set of hex points
pub type HexPointSet = HashSet<HexPoint, FnvBuildHasher>;
/// A map of hex points to some `T`
pub type HexPointMap<T> = HashMap<HexPoint, T, FnvBuildHasher>;
/// An ORDERED map of hex points to some `T`. This has some extra memory
/// overhead, so we should only use it when we actually need the ordering.
pub type HexPointIndexMap<T> = IndexMap<HexPoint, T, FnvBuildHasher>;

/// A cluster is a set of contiguous hex points. All items in a cluster are
/// adjacent to at least one other item in the cluster (unless the cluster is a
/// singular item).
#[derive(Clone, Debug)]
pub struct Cluster<T> {
    tiles: HexPointIndexMap<T>,
    adjacents: HexPointSet,
}

impl<T: Debug> Cluster<T> {
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

    /// Add a new tile to the cluster
    pub fn insert(&mut self, pos: HexPoint, tile: T) {
        // Any tile we add in should already be a neighbor of the cluster. If
        // it isn't that means it's discontiguous which breaks the cardinal rule
        if !self.adjacents.remove(&pos) {
            panic!(
                "Cannot add tile at {} to cluster {:?}, it is not adjacent!",
                pos, self
            );
        }

        // Add the tile to the map, and add its neighbors to our set of
        // neighbors
        self.tiles.insert(pos, tile);
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
        // TODO make this more efficient somehow
        self.adjacents = self
            .adjacents
            .iter()
            .copied()
            .chain(other.adjacents.into_iter())
            .filter(|pos| !self.tiles.contains_key(pos))
            .collect();
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
#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum HexDirection {
    Up,
    UpRight,
    DownRight,
    Down,
    DownLeft,
    UpLeft,
}

impl HexDirection {
    /// Get an vector offset that would move a point one tile in this direction
    pub fn vec(self) -> HexVec {
        match self {
            Self::Up => HexVec::new(0, 1),
            Self::UpRight => HexVec::new(1, 0),
            Self::DownRight => HexVec::new(1, -1),
            Self::Down => HexVec::new(0, -1),
            Self::DownLeft => HexVec::new(-1, 0),
            Self::UpLeft => HexVec::new(-1, 1),
        }
    }
}

/// The 3 axes in our coordinate system.
///
/// See this page for more info (we use "flat topped" tiles):
/// https://www.redblobgames.com/grids/hexagons/#coordinates-cube
#[derive(Copy, Clone, Debug, EnumIter)]
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
#[derive(Copy, Clone, Debug)]
pub struct HexAxialDirection {
    pub axis: HexAxis,
    pub positive: bool,
}

impl HexAxialDirection {
    pub fn signum(self) -> i16 {
        if self.positive {
            1
        } else {
            -1
        }
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
    fn test_world_size() {
        assert_eq!(<WorldMap<()>>::world_size(0), 1);
        assert_eq!(<WorldMap<()>>::world_size(1), 7);
        assert_eq!(<WorldMap<()>>::world_size(2), 19);
        assert_eq!(<WorldMap<()>>::world_size(3), 37);
    }

    #[test]
    fn test_world_map_len() {
        assert_eq!(WorldMap::new(0, |_| ()).len(), 1);
        assert_eq!(WorldMap::new(1, |_| ()).len(), 7);
        assert_eq!(WorldMap::new(2, |_| ()).len(), 19);
        assert_eq!(WorldMap::new(3, |_| ()).len(), 37);
    }
}
