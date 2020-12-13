use std::collections::VecDeque;

use crate::world::{
    hex::{HasHexPosition, HexPoint, HexPointMap},
    tile::{Tile, TileMap},
};

/// A cluster is a set of contiguous hex points. All tiles in a cluster are
/// adjacent to at least one other tile in the cluster (unless the cluster is a
/// singular tile).
pub struct Cluster(HexPointMap<()>);

impl Cluster {
    pub fn make_predicate_clusters<P: Fn(&Tile) -> bool>(
        predicate: P,
        tiles: TileMap,
    ) -> Vec<Self> {
        // Here's our algorithm:
        // - Create a pool of tiles that have yet to be clustered
        // - Grab a random tile from the pool
        // - If it matches the predicate, do a BFS out from that tile, including
        //   all tiles that match the predicate
        // - Once we run out of matchings tiles, consider the cluster complete
        // - Repeat with the remaining unclustered tiles

        // This map is all the tiles that we've yet to check
        let mut unclustered_tiles = tiles;
        let mut clusters = Vec::new();

        // Grab the first unchecked tile and start building a cluster around it.
        // This loop runs once per generated cluster, plus once per each failed
        // attempt at a cluster (where the first tile fails the predicate)
        while let Some((_, first_tile)) = unclustered_tiles.pop_first() {
            // Start our BFS. We'll use a queue of the next tiles to check, and
            // seed it with our first tile.
            let mut tile_queue: VecDeque<Tile> =
                VecDeque::with_capacity(unclustered_tiles.len());
            tile_queue.push_back(first_tile);
            let mut cluster = HexPointMap::new();

            // Grab the next tile off the queue and check it
            while let Some(tile) = tile_queue.pop_front() {
                if predicate(&tile) {
                    // If it passes the pred, then add it to the cluster and add
                    // its neighbors to the queue
                    cluster.insert(tile.position(), ());
                    // Remove all the adjacent tiles from the map and add them
                    // to the queue
                    tile_queue.extend(
                        unclustered_tiles.take_adjacents(tile.position()),
                    );
                }
            }

            if cluster.len() > 0 {
                clusters.push(Cluster(cluster));
            }
        }

        clusters
    }
}
