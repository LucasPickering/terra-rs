use crate::world::hex::{TileDirection, TilePoint};
use fnv::FnvBuildHasher;
use indexmap::{map::Entry, IndexMap};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};

/// A set of tile points
pub type TilePointSet = HashSet<TilePoint, FnvBuildHasher>;
/// A map of tile points to some `T`
pub type TilePointMap<T> = HashMap<TilePoint, T, FnvBuildHasher>;
/// An ORDERED map of tile points to some `T`. This has some extra memory
/// overhead, so we should only use it when we actually need the ordering.
pub type TilePointIndexMap<T> = IndexMap<TilePoint, T, FnvBuildHasher>;
/// A map of tile directions to some `T`
pub type TileDirectionMap<T> = HashMap<TileDirection, T, FnvBuildHasher>;

/// A static mapping of hex directions to values. This is similar to a
/// `HexDirectionMap<T>`, except that it always holds exactly 6 values and they
/// can be accessed via static fields. This is more useful in some cases,
/// especially post-world generation because we won't have to add or remove
/// values at that point. Having static fields makes serialization in external
/// apps a bit easier.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TileDirectionValues<T: Copy + Clone + Debug + PartialEq + Serialize>
{
    pub northeast: T,
    pub east: T,
    pub southeast: T,
    pub southwest: T,
    pub west: T,
    pub northwest: T,
}

impl<T: Copy + Clone + Debug + PartialEq + Serialize> TileDirectionValues<T> {
    /// Copy all values in this struct into an array. The ordering will be the
    /// same as the iteration order of [HexDirection]: Clockwise, starting with
    /// `up`.
    pub fn as_array(&self) -> [T; 6] {
        [
            self.northeast,
            self.east,
            self.southeast,
            self.southwest,
            self.west,
            self.northwest,
        ]
    }
}

// Convert a dynamic map into a struct with fixed fields. Missing values will
// be populated with defaults
impl<T: Copy + Clone + Debug + Default + PartialEq + Serialize>
    From<TileDirectionMap<T>> for TileDirectionValues<T>
{
    fn from(mut map: TileDirectionMap<T>) -> Self {
        let mut helper =
            |dir: TileDirection| map.remove(&dir).unwrap_or_default();

        Self {
            // NORTH
            northeast: helper(TileDirection::NNE),
            // EASTERN
            east: helper(TileDirection::E),
            southeast: helper(TileDirection::SSE),
            southwest: helper(TileDirection::SSW),
            west: helper(TileDirection::W),
            northwest: helper(TileDirection::NNW),
        }
    }
}

/// A cluster is a set of contiguous tile points. All items in a cluster are
/// adjacent to at least one other item in the cluster (unless the cluster is a
/// singular item).
#[derive(Clone, Debug)]
pub struct Cluster<T> {
    tiles: TilePointIndexMap<T>,
    adjacents: TilePointSet,
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
        tiles: &mut TilePointMap<T>,
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
        let mut remaining: TilePointIndexMap<&mut T> =
            tiles.iter_mut().map(|(pos, t)| (*pos, t)).collect();
        let mut clusters: Vec<Cluster<&mut T>> = Vec::new();

        // Grab the first unchecked item and start building a cluster around it.
        // This loop runs once per generated cluster, plus once per each failed
        // attempt at a cluster (where the first item fails the predicate)
        while let Some(first_entry) = remaining.pop() {
            let mut cluster = TilePointIndexMap::default();
            // Start our BFS. We'll use a queue of the next items to check, and
            // seed it with our first item. It doesn't seem to matter if we
            // allocate this on each loop or do it outside, so it's probably
            // getting optimized to the same thing
            let mut bfs_queue: VecDeque<(TilePoint, &mut T)> = VecDeque::new();
            // Start with this tile - if it fails the predicate it'll be the
            // only one we check for this cluster
            bfs_queue.push_back(first_entry);

            // Grab the next item off the queue and check it
            while let Some((pos, item)) = bfs_queue.pop_front() {
                if predicate(item) {
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

    pub fn new(tiles: TilePointIndexMap<T>) -> Self {
        // Initialize the set of all tiles that are adjacent to (but not in) the
        // cluster
        let mut adjacents = TilePointSet::default();
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
    pub fn tiles(&self) -> &TilePointIndexMap<T> {
        &self.tiles
    }

    /// Move the tile map out of this struct
    pub fn into_tiles(self) -> TilePointIndexMap<T> {
        self.tiles
    }

    /// The set of positions that are directly adjacent to at least one tile in
    /// this cluster, but NOT in the cluster themselves. **These positions do
    /// not necessarily exist in the world!** The cluster has no context of the
    /// bounds of the world, so there's no guarantees around the validity of
    /// these neighbors.
    pub fn adjacents(&self) -> &TilePointSet {
        &self.adjacents
    }

    /// Add a new tile to the cluster. Returns an error if there is already a
    /// tile at that position, or if the new tile isn't contiguous to the
    /// cluster.
    pub fn insert(&mut self, pos: TilePoint, tile: T) {
        // Any tile we add in should already be a neighbor of the cluster. If
        // it isn't that means it's discontiguous which breaks the cardinal rule
        let removed = self.adjacents.remove(&pos);
        assert!(
            removed,
            "cannot add tile at {pos} to cluster {self:?}, it is not adjacent!",
        );

        // Add the tile to the map, and add its neighbors to our set of
        // neighbors
        match self.tiles.entry(pos) {
            Entry::Vacant(entry) => {
                entry.insert(tile);
            }
            Entry::Occupied(_) => {
                panic!("tile {pos} is already in cluster {self:?}");
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
    fn test_hex_direction_map_to_values() {
        // Make sure each direction is mapped correctly
        let mut map: TileDirectionMap<usize> = TileDirectionMap::default();
        map.insert(TileDirection::NNE, 1);
        map.insert(TileDirection::E, 2);
        map.insert(TileDirection::SSE, 3);
        map.insert(TileDirection::SSW, 4);
        map.insert(TileDirection::W, 5);
        map.insert(TileDirection::NNW, 6);
        assert_eq!(
            TileDirectionValues::from(map),
            TileDirectionValues {
                northeast: 1,
                east: 2,
                southeast: 3,
                southwest: 4,
                west: 5,
                northwest: 6,
            }
        );

        // Should populate missing fields with the default
        assert_eq!(
            TileDirectionValues::from(TileDirectionMap::default()),
            TileDirectionValues {
                northeast: 0,
                east: 0,
                southeast: 0,
                southwest: 0,
                west: 0,
                northwest: 0,
            }
        );
    }
}
