use derive_more::Display;
use std::{
    collections::{BTreeMap, VecDeque},
    iter::FromIterator,
    ops::{self, Deref, DerefMut},
};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Display)]
#[display(fmt = "({}, {}, {})", x, y, z)]
pub struct HexPoint {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl HexPoint {
    /// Construct a new hex point with the given x and y. Since x+y+z=0 for all
    /// points, we can derive z from x & y.
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y, z: -(x + y) }
    }

    /// Convert this hexagonal coordinate into a 2d pixel coordinate. Useful
    /// for figuring out where to position a tile on the screen. Tiles are
    /// rendered flat-top, meaning the x axis in hex coords aligns with the x
    /// axis in 3D. That means all tiles on the 3D-z axis have a hex-x value of
    /// 0. https://www.redblobgames.com/grids/hexagons/#coordinates-cube
    pub fn pixel_pos(&self, scale: f32) -> (f32, f32) {
        let pixel_x: f32 = self.x as f32 * 0.75;
        let pixel_z: f32 =
            (self.x as f32 / 2.0 + self.y as f32) * -(3.0_f32.sqrt() / 2.0);
        (pixel_x * scale, pixel_z * scale)
    }
}

impl ops::Add<HexPoint> for HexPoint {
    type Output = HexPoint;

    fn add(self, rhs: HexPoint) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/// A map of hex-positioned items, keyed by their position.
#[derive(Clone, Debug, Default)]
pub struct HexPointMap<T> {
    map: BTreeMap<HexPoint, T>,
}

impl<T> HexPointMap<T> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    /// Find all the items adjacent to the given position. This can return up to
    /// 6 items, but will return less if there are gaps in the map or the
    /// position is at the edge.
    pub fn adjacents(
        &self,
        pos: HexPoint,
    ) -> impl Iterator<Item = (HexPoint, &T)> {
        HexDirection::iter().filter_map(move |dir| {
            let adj_pos = pos + dir.offset();
            Some((adj_pos, self.map.get(&adj_pos)?))
        })
    }

    /// Find all the items adjacent to the given position, remove them from the
    /// map, and return them. This can return up to 6 items, but will return
    /// less if there are gaps in the map or the position is at the edge.
    pub fn take_adjacents(&mut self, pos: HexPoint) -> Vec<(HexPoint, T)> {
        HexDirection::iter()
            .filter_map(move |dir| {
                let adj_pos = pos + dir.offset();
                self.map.remove_entry(&adj_pos)
            })
            .collect()
    }

    /// Locate clusters of points within this map according to a predicate. All
    /// items that satisfy the predicate will be clustered such that any two
    /// satisfactory tiles that are adjacent to each other will be in a cluster
    /// together. The returned cluster holds mutables references to the
    /// items in this map, so they can be modified after clustering.
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
            self.iter_mut().map(|(pos, item)| (*pos, item)).collect();
        let mut clusters: Vec<Cluster<&mut T>> = Vec::new();

        // Grab the first unchecked item and start building a cluster around it.
        // This loop runs once per generated cluster, plus once per each failed
        // attempt at a cluster (where the first item fails the predicate)
        while let Some(first_entry) = remaining.pop_first() {
            let mut cluster = HexPointMap::new();
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
                    // Remove all the adjacent items from the map and add them
                    // to the queue
                    bfs_queue.extend(remaining.take_adjacents(pos));
                }
            }

            if cluster.len() > 0 {
                clusters.push(Cluster(cluster));
            }
        }

        clusters
    }
}

// For iterators of pairs
impl<T> FromIterator<(HexPoint, T)> for HexPointMap<T> {
    fn from_iter<I: IntoIterator<Item = (HexPoint, T)>>(iter: I) -> Self {
        Self {
            map: iter.into_iter().collect(),
        }
    }
}

// Shortcut for iterators of items that hold a position internally
impl<T: HasHexPosition> FromIterator<T> for HexPointMap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter()
            .map(|item| (item.position(), item))
            .collect()
    }
}

// Shortcut for a map of unit types, which is just a set of points
impl FromIterator<HexPoint> for HexPointMap<()> {
    fn from_iter<I: IntoIterator<Item = HexPoint>>(iter: I) -> Self {
        iter.into_iter().map(|pos| (pos, ())).collect()
    }
}

impl<T> IntoIterator for HexPointMap<T> {
    type Item = <BTreeMap<HexPoint, T> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<HexPoint, T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<T> Deref for HexPointMap<T> {
    type Target = BTreeMap<HexPoint, T>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<T> DerefMut for HexPointMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

/// A cluster is a set of contiguous hex points. All items in a cluster are
/// adjacent to at least one other item in the cluster (unless the cluster is a
/// singular item).
pub struct Cluster<T>(pub HexPointMap<T>);

pub trait HasHexPosition: Sized {
    fn position(&self) -> HexPoint;

    /// Convert this value into a tuple with the position. Useful when mapping
    /// an iterator then collecting into a [HexPointMap].
    fn into_tuple(self) -> (HexPoint, Self) {
        (self.position(), self)
    }
}

/// The 6 directions on the hex axes. Left/right is aligned with the x axis
#[derive(Copy, Clone, Debug, EnumIter)]
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
