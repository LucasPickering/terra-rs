use derive_more::{Add, Display};
use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet};
use std::{collections::VecDeque, convert::TryInto, fmt::Debug, hash::Hash};
use strum::{EnumIter, IntoEnumIterator};
use wasm_bindgen::prelude::*;

/// A point in a hexagon-tiled world. Each point has an x, y, and z component.
/// See this page for info on how the 3D system works:
/// https://www.redblobgames.com/grids/hexagons/
///
/// This struct actually only needs to store x and y, since x+y+z=0 for all
/// points, so z can be derived as necessary which means we can save 33% of
/// the memory.
///
/// The x and y coordinates are stored as i16s. We'll never have a world with
/// a radius of more than 32k (that'd be ~4 billion tiles), so this saves on
/// memory a lot.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Display, Add)]
#[display(fmt = "({}, {}, {})", "self.x()", "self.y()", "self.z()")]
pub struct HexPoint {
    x: i16,
    y: i16,
}

#[wasm_bindgen]
impl HexPoint {
    /// Construct a new hex point with the given x and y. Since x+y+z=0 for all
    /// points, we can derive z from x & y.
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> i16 {
        self.x
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> i16 {
        self.y
    }

    #[wasm_bindgen(getter)]
    pub fn z(&self) -> i16 {
        -(self.x + self.y)
    }
}

impl HexPoint {
    /// Get an iterator of all the points directly adjacent to this one. The
    /// iterator will always contain exactly 6 values.
    pub fn adjacents(self) -> impl Iterator<Item = HexPoint> {
        HexDirection::iter().map(move |dir| self + dir.offset())
    }
}

/// A map of items keyed by hex point. There is a major restriction on this
/// though: it has to be a full world of items, meaning it must be a "square"!
/// This restriction allows us to optimize a lot by storing all the items in
/// a flat vector.
#[derive(Clone, Debug, Default)]
pub struct WorldMap<T> {
    /// Distance from the center of the world to the edge. 0 means the world is
    /// exactly 1 tile. 1 means 7 tiles, and so on. We frequently need to treat
    /// this as an `i16` as well, so this cannot exceed [std::isize::MAX]!
    /// If it does, we're gonna have bigger problems anyway...
    world_radius: u16,
    /// Items get flattened into a vec. We can do this because we know the
    /// exact x/y bounds of the world. Items are ordered by ascending y, then
    /// ascending x. E.g. [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 0), ...]
    vec: Vec<T>,
}

impl<T> WorldMap<T> {
    /// Initialize a new world with the given radius. The passed function will
    /// be called to initialize each item in the world.
    pub fn new(world_radius: u16, initializer: impl Fn(HexPoint) -> T) -> Self {
        // We'll always have (2r + 1)^2 tiles, because the range for x and y
        // is [-r, r]
        let x: usize = 2 * (world_radius as usize) + 1;
        let capacity = x * x;
        let mut vec = Vec::with_capacity(capacity as usize);

        // Initialize a set of tiles with no data
        let radius = world_radius as i16;
        for x in -radius..=radius {
            for y in -radius..=radius {
                // x+y+z == 0 always, so we can derive z from x & y.
                let pos = HexPoint::new(x, y);
                vec.push(initializer(pos));
            }
        }

        Self { world_radius, vec }
    }

    /// Get the number of items on one side of the world. This number squared
    /// will be the total number of items in the vec.
    fn world_dim_len(&self) -> usize {
        2 * (self.world_radius as usize) + 1
    }

    /// Convert the given position to an index into our vec, for doing internal
    /// lookups. Returns `Some` iff the position is in the map, `None` if it
    /// produces an invalid index.
    fn pos_to_index(&self, pos: HexPoint) -> Option<usize> {
        // Shift x/y to be zero-based, i.e. do this:
        // [radius=10] (-5, 5) -> (5, 15)
        // If the position is not in the map, then one of these converions may
        // fail. In that case, do a quick getaway.
        let x: usize = (pos.x() + self.world_radius as i16).try_into().ok()?;
        let y: usize = (pos.y() + self.world_radius as i16).try_into().ok()?;

        // [radius=10] (5, 15) -> (5 * 21) + 15 -> 120
        let idx = (x * self.world_dim_len()) + y;
        // Make sure the index is valid.
        if idx < self.vec.len() {
            Some(idx)
        } else {
            None
        }
    }

    pub fn get(&self, pos: HexPoint) -> Option<&T> {
        self.vec.get(self.pos_to_index(pos)?)
    }

    pub fn get_mut(&mut self, pos: HexPoint) -> Option<&mut T> {
        let idx = self.pos_to_index(pos)?;
        self.vec.get_mut(idx)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.vec.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.vec.iter_mut()
    }

    /// Map this collection into a new collection by apply the given mapping
    /// function over each element. We need this instead of just relying on
    /// [std::iter::Iterator::map] because you can't collect an iterator into a
    /// [WorldMap].
    pub fn map<U>(self, f: impl Fn(T) -> U) -> WorldMap<U> {
        let vec = self.vec.into_iter().map(f).collect();
        WorldMap {
            world_radius: self.world_radius,
            vec,
        }
    }

    /// Get the number of items in the map
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Find all the items adjacent to the given position. This can return up to
    /// 6 items, but will return less if there are gaps in the map or the
    /// position is at the edge.
    pub fn adjacents(&self, pos: HexPoint) -> impl Iterator<Item = &T> {
        pos.adjacents().filter_map(move |adj| self.get(adj))
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
        let mut remaining: HexPointMap<&mut T> =
            self.iter_mut().map(|t| (t.position(), t)).collect();
        let mut clusters: Vec<Cluster<&mut T>> = Vec::new();

        // Grab the first unchecked item and start building a cluster around it.
        // This loop runs once per generated cluster, plus once per each failed
        // attempt at a cluster (where the first item fails the predicate)
        while let Some(first_entry) = remaining.pop() {
            let mut cluster = HexPointMap::default();
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
    type Item = <Vec<T> as IntoIterator>::Item;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

pub type HexPointSet = IndexSet<HexPoint, FnvBuildHasher>;
pub type HexPointMap<T> = IndexMap<HexPoint, T, FnvBuildHasher>;

/// A cluster is a set of contiguous hex points. All items in a cluster are
/// adjacent to at least one other item in the cluster (unless the cluster is a
/// singular item).
#[derive(Clone, Debug)]
pub struct Cluster<T> {
    pub tiles: HexPointMap<T>,
    pub adjacents: HexPointSet,
}

impl<T: Debug> Cluster<T> {
    pub fn new(tiles: HexPointMap<T>) -> Self {
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

    /// The set of positions that are directly adjacent to at least one tile in
    /// this cluster, but NOT in the cluster themselves. **These positions do
    /// not necessarily exist in the world!** The cluster has no context of the
    /// bounds of the world, so there's no guarantees around the validity of
    /// these neighbors.
    pub fn adjacents(&self) -> &HexPointSet {
        &self.adjacents
    }

    /// TODO
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
}

pub trait HasHexPosition: Sized {
    fn position(&self) -> HexPoint;
}

/// The 6 directions on the hex axes. Left/right is aligned with the x axis
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
    /// Get an offset that would move a point in this direction
    pub fn offset(self) -> HexPoint {
        match self {
            Self::Up => HexPoint::new(0, 1),
            Self::UpRight => HexPoint::new(1, 0),
            Self::DownRight => HexPoint::new(1, -1),
            Self::Down => HexPoint::new(0, -1),
            Self::DownLeft => HexPoint::new(-1, 0),
            Self::UpLeft => HexPoint::new(-1, 1),
        }
    }
}
