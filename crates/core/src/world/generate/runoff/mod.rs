mod basin;
mod pattern;

use crate::{
    util::{self, Meter, Meter3},
    world::{
        generate::{
            runoff::{
                basin::{Basin, Basins},
                pattern::{RunoffDestination, RunoffPattern},
            },
            Generate, TileBuilder, WorldBuilder,
        },
        hex::{
            Cluster, HasHexPosition, HexDirection, HexPoint, HexPointIndexMap,
            HexPointMap,
        },
        Tile, World,
    },
};
use fnv::FnvBuildHasher;
use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    default::Default,
    iter,
};
use strum::IntoEnumIterator;

/// Simulate water runoff. This applies some amount of rainfall to each tile,
/// then simulates the water flowing downhill. This doesn't actually do
/// anything with the runoff values though, it just sets them. A separate
/// generator is responsible for turning the runoff values into
/// lakes/rivers/etc.
///
/// This needs to run AFTER ocean generation!
#[derive(Debug)]
pub struct RunoffGenerator;

impl Generate for RunoffGenerator {
    fn generate(&self, world: &mut WorldBuilder) {
        let continents =
            Cluster::predicate(&mut world.tiles, |tile| !tile.is_water());
        // Hypothetically we could run these simulations in parallel since each
        // continent is independent, but skipping that for now cause Wasm.
        for continent in continents {
            let mut continent = Continent::new(continent.into_tiles());
            continent.sim_continent_runoff();
        }
    }
}

/// Compare two tiles by their elevation
fn cmp_elev(a: &TileBuilder, b: &TileBuilder) -> Ordering {
    util::cmp_unwrap(&a.elevation().unwrap(), &b.elevation().unwrap())
}

struct Continent<'a> {
    /// All the tiles in this continent. After continent creation, this will
    /// not be added to or removed from, but it may be reoredered and the
    /// individual tiles may be mutated.
    tiles: HexPointIndexMap<&'a mut TileBuilder>,
    /// The runoff pattern of every tile in this continent. Once initialized,
    /// this map will corresponding 1:1 with `self.tiles`. That means they will
    /// have the same length **and** the same ordering. This makes lookups
    /// & iterating easier in some scenarios because we can zip the two
    /// together or do cross lookups based on index instead of key.
    runoff_patterns: HexPointIndexMap<RunoffPattern>,
}

impl<'a> Continent<'a> {
    fn new(mut tiles: HexPointIndexMap<&'a mut TileBuilder>) -> Self {
        let runoff_patterns = Self::calc_runoff_patterns(&mut tiles);
        Self {
            tiles,
            runoff_patterns,
        }
    }

    /// For each tile, calculate its runoff pattern. This pattern makes it easy
    /// to push runoff around later. Every tile in the continent will get a
    /// pattern, so the length of the output will match the length of the
    /// input. The output will be a map with all the same tiles as the
    /// input, with each tile paired to its runoff pattern.
    ///
    /// **This will reorder the input!** The continent needs to be sorted by
    /// ascending elevation to calculate runoff patterns.
    fn calc_runoff_patterns(
        tiles: &mut HexPointIndexMap<&mut TileBuilder>,
    ) -> HexPointIndexMap<RunoffPattern> {
        // Sort tiles by ascending elevation. This is very important! Runoff
        // patterns have to be generated low->high so the patterns of their
        // lower neighbors. Once we have a pattern for each tile, we can
        // easily calculate where water ends up for each tile.
        tiles.sort_by(|_, a, _, b| cmp_elev(a, b));

        // Build a map of runoff patterns for each tile. IMPORTANT: this map has
        // the same ordering as self.tiles, which allows us to do index lookups
        // instead of key lookups later. gotta go fast
        let mut runoff_patterns = HexPointIndexMap::default();
        for source_tile in tiles.values() {
            // For each neighbor of this tile, determine how much water it gets.
            // This is a map of direction:elevation_diff
            let recipients: Vec<(HexDirection, Meter)> = HexDirection::iter()
                .filter_map(|dir| {
                    let adj_pos = source_tile.position() + dir.vec();
                    let adj_elev = match tiles.get(&adj_pos) {
                        // Adjacent tile isn't part of this continent, so assume
                        // it's ocean
                        None => World::SEA_LEVEL,
                        Some(adj_tile) => adj_tile.elevation().unwrap(),
                    };
                    let elev_diff = source_tile.elevation().unwrap() - adj_elev;
                    if elev_diff > Meter(0.0) {
                        // Neighbor is lower, we'll send runoff there
                        Some((dir, elev_diff))
                    } else {
                        // Neighbor is higher, ignore it
                        None
                    }
                })
                .collect();

            // Distribute the water to our neighbors. Each neighbor gets an
            // amount proportional to the elevation different
            // between us and them. I.e. steeper slopes get more
            // water.

            let total_elev_diff: Meter =
                recipients.iter().map(|(_, elev_diff)| *elev_diff).sum();

            // For each adjacent lower tile, mark it as an exit in the pattern
            let mut runoff_pattern = RunoffPattern::new(source_tile.position());
            for (dir, elev_diff) in recipients {
                let adj_pos = source_tile.position() + dir.vec();
                runoff_pattern.add_exit(
                    dir,
                    // This is why the tiles have to be ascending by elevation,
                    // because we back-reference the lower tiles
                    runoff_patterns.get(&adj_pos),
                    elev_diff.0 / total_elev_diff.0,
                );
            }
            runoff_patterns.insert(source_tile.position(), runoff_pattern);
        }

        runoff_patterns
    }

    /// Simulate runoff for a single continent. Each continent is an independent
    /// system, meaning its runoff doesn't affect any other continents in any
    /// way.
    fn sim_continent_runoff(&mut self) {
        self.initialize_runoff();
        self.push_downhill();
        self.sim_backflow();
    }

    /// Generate an initial runoff level for every tile in a continent.
    fn initialize_runoff(&mut self) {
        // Set initial runoff for each tile
        for tile in self.tiles.values_mut() {
            // Set initial runoff level
            tile.set_runoff(tile.rainfall().unwrap());
        }
    }

    /// Push all runoff on the continent downhill, so that it all ends up in
    /// just two places: terminal tiles and the ocean. Must runoff will end
    /// up in the ocean, but basins inside the continent will collect runoff
    /// at the terminals. Each dip will only have a single terminal though,
    /// so after this step we will still need to simulate "backflow".
    fn push_downhill(&mut self) {
        // Now that we have our runoff patterns, we can figure out how much
        // water ends up going to each terminal. We have to do this in
        // two steps because borrow checking.
        let mut terminal_runoffs: HexPointMap<Meter3> = HexPointMap::default();
        for (source_tile, source_pattern) in
            self.tiles.values_mut().zip(self.runoff_patterns.values())
        {
            let to_distribute = source_tile.clear_runoff();
            // Add the appropriate amount of water to each terminal. For most
            // tiles, the terminals' factors don't add up to 1, so
            // some or all of the water gets deleted from the system
            // (i.e. flows to the ocean).
            for (term_pos, factor) in source_pattern.terminals().iter() {
                match term_pos {
                    None => {
                        // This runoff flows to the ocean, so nothing to do here
                    }
                    Some(term_pos) => {
                        *terminal_runoffs
                            .entry(*term_pos)
                            .or_insert(Meter3(0.0)) += to_distribute * factor;
                    }
                }
            }
        }
        for (pos, runoff) in terminal_runoffs {
            self.tiles.get_mut(&pos).unwrap().add_runoff(runoff);
        }
    }

    /// Simulate "backflow", which is when runoff that has collected on a
    /// terminal tile spreads out to its neighbors. In some cases, the
    /// runoff on the terminal can be neatly distributed in its area, but in
    /// some cases it will overflow the terminal's basin, and some of it
    /// will end up flowing over into the ocean. We also need to handle
    /// cases where two terminal clusters join to form a larger lake, or
    /// when one cluster overflows into another but they DON'T join.
    fn sim_backflow(&mut self) {
        // For each terminal, map it to its constituents (all the other tiles
        // that it will spread to)
        let mut basins = Basins::new(&mut self.tiles);

        let mut basin_queue: VecDeque<HexPoint> = basins.keys().collect();
        while let Some(basin_key) = basin_queue.pop_front() {
            let basin = basins.get_mut(basin_key).unwrap();
            let overflow_distribution = self.grow_basin(basin);

            // If this basin overflowed into other(s), then do some processing
            // for each one
            for (overflow_dest, overflow_vol) in overflow_distribution {
                // If the overflow destination is a terminal tile (as opposed to
                // ocean), then push the overflow runoff into that basin
                if let Some(other_basin_key) = overflow_dest {
                    if basins
                        .has_previously_overflowed(other_basin_key, basin_key)
                    {
                        // This other basin has already donated to us. Since
                        // we've overflowed in both directions now, the two need
                        // to be joined
                        let joined_basin = basins.join(
                            basin_key,
                            other_basin_key,
                            overflow_vol,
                        );

                        // Rebuild the queue to exclude any terminals in the
                        // newly created basin, then queue the primary key for
                        // that basin
                        basin_queue = basin_queue
                            .into_iter()
                            .filter(|pos| {
                                !joined_basin.terminals().contains(pos)
                            })
                            .chain(iter::once(joined_basin.key()))
                            .collect();
                    } else {
                        // The other basin has never donated to us, which means
                        // we can safely overflow into them
                        let other_basin =
                            basins.get_mut(other_basin_key).unwrap();
                        other_basin.overflow(basin_key, overflow_vol);

                        // Re-queue the receiving basin (if it isn't already)
                        if !basin_queue.contains(&other_basin_key) {
                            basin_queue.push_back(other_basin_key);
                        }
                    }
                }
            }
        }

        // Now we know that each cluster is finalized, we can distribute the
        // runoff accordingly
        for basin in basins.into_basins() {
            let runoff_elev = basin.runoff_elevation();
            for pos in basin.tiles().tiles().keys() {
                let tile = self.tiles.get_mut(pos).unwrap();
                let runoff_height = runoff_elev - tile.elevation().unwrap();
                // Convert Meter -> Meter3
                tile.set_runoff(runoff_height * Tile::AREA);
            }
        }
    }

    /// Spread around runoff for a single "basin". A basin is one cluster that
    /// grows out from a singular terminal tile. This will start the cluster
    /// off as just that tile, and will grow it out until we either:
    ///
    /// 1. Run out of water to keep spreading
    /// 2. Overflow into the ocean/another basin
    ///
    /// The return value is the amount of runoff that has overflowed, and the
    /// target(s) that it's overflowed to. The sum of the return map's values
    /// will be 1.0 **iff** it is not empty. If it *is* empty, that means we
    /// didn't overflow at all.
    fn grow_basin(
        &self,
        basin: &mut Basin,
    ) -> HashMap<RunoffDestination, Meter3, FnvBuildHasher> {
        // Ok so here's the deal: We have a single terminal tile with a bunch of
        // runoff on it, and we need to distribute it around. The general
        // approach is:
        // 1. Find the lowest neighbor to the basin
        // 2. See if we have enough runoff to overflow onto that neighbor
        //   a. If so, then overflow onto it and repeat from step 1
        //   b. If not, then our cluster is complete

        // Each iteration of this loop will add a tile to the cluster, EXCEPT
        // for the last iteration. So for n iterations, we add n-1 tiles. This
        // loop will ALWAYS run at least once. In order for it not to, we'd have
        // to have a tile that is (1) a terminal and (2) has no land neighbors,
        // which doesn't make any sense.
        while let Some(candidate_tile) = basin
            .tiles()
            .adjacents()
            .iter()
            .filter_map(|pos| self.tiles.get(pos))
            .min_by(|a, b| cmp_elev(a, b))
        {
            // Just a sanity check. We expect every tile that's not a terminal
            // to have no runoff on it. (and all terminals are initialized to
            // be in a basin, so none of them should ever become candidates).
            assert!(
                candidate_tile.runoff().unwrap() == Meter3(0.0),
                "All candidates should have 0 runoff"
            );

            // If the candidate is higher than our current water level, then
            // we can't reach it so the runoff stops spreading.
            let candidate_elev = candidate_tile.elevation().unwrap();
            if candidate_elev >= basin.runoff_elevation() {
                break;
            }

            // If the candidate drains, totally or partially, somewhere else
            // (another basin(s) and/or the ocean), then we need to overflow
            // into those other target(s) before we can grow this basin anymore.
            // You can think of this as the water level of the basin rising up
            // until it starts to leak out at the lowest opening.
            let candidate_pattern = self
                .runoff_patterns
                .get(&candidate_tile.position())
                .unwrap();
            let overflow_vol: Meter3 = (basin.runoff_elevation()
                - candidate_elev)
                * Tile::AREA
                * (basin.tiles().tiles().len() as f64);
            let overflow_distribution =
                basin.distribute_elsewhere(candidate_pattern, overflow_vol);
            if !overflow_distribution.is_empty() {
                return overflow_distribution;
            }

            basin.add_tile(candidate_tile); // Initiation!
        }

        HashMap::default() // This won't allocate for an empty map
    }
}
