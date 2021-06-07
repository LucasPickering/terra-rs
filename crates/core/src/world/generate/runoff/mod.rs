mod basin;
mod pattern;

pub use crate::world::generate::runoff::pattern::RunoffPattern;
use crate::{
    unwrap,
    util::{
        self,
        unit::{Meter, Meter3},
    },
    world::{
        generate::{
            runoff::{
                basin::{Basin, Basins},
                pattern::RunoffDestination,
            },
            Generate, TileBuilder, WorldBuilder,
        },
        hex::{
            Cluster, HasHexPosition, HexDirection, HexPoint, HexPointIndexMap,
        },
        Tile, World,
    },
};
use assert_approx_eq::assert_approx_eq;
use fnv::FnvBuildHasher;
use log::trace;
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
        // If rainfall sim is disabled then there won't be any runoff to
        // simulate, so we might as well skip this step too
        if world.config.rainfall.enabled {
            let continents = Cluster::predicate(&mut world.tiles, |tile| {
                !tile.is_water_biome()
            });
            // Hypothetically we could run these simulations in parallel since
            // each continent is independent, but skipping that for
            // now cause Wasm.
            for continent in continents {
                let mut continent = Continent::new(continent.into_tiles());
                continent.sim_continent_runoff();
            }
        } else {
            for tile in world.tiles.values_mut() {
                tile.set_runoff(Meter3(0.0));
            }
        }
    }
}

/// Compare two tiles by their elevation
fn cmp_elev(a: &TileBuilder, b: &TileBuilder) -> Ordering {
    util::cmp_unwrap(&a.elevation(), &b.elevation())
}

/// A cluster of land tiles. One tile cannot belong to more than one continent.
struct Continent<'a> {
    /// A point that unique identifies this continent. This point should be any
    /// tile within the continent, since each tile cannot belong to any other
    /// continent. Exactly which tile it is isn't important, since it isn't
    /// used for calculations, just as a unique ID.
    id: HexPoint,
    /// All the tiles in this continent. After continent creation, this will
    /// not be added to or removed from. **These tiles will be sorted by
    /// ascending elevation**. Individual tiles can be mutated as well.
    tiles: HexPointIndexMap<&'a mut TileBuilder>,
}

impl<'a> Continent<'a> {
    fn new(mut tiles: HexPointIndexMap<&'a mut TileBuilder>) -> Self {
        let (&id, _) =
            tiles.first().expect("cannot initialize empty continent");

        // Sort tiles by ascending elevation. This is very important! Runoff
        // patterns have to be generated low->high so the patterns of their
        // lower neighbors. Once we have a pattern for each tile, we can
        // easily calculate where water ends up for each tile.
        tiles.sort_by(|_, a, _, b| cmp_elev(a, b));

        Self { id, tiles }
    }

    /// Simulate runoff for a single continent. Each continent is an independent
    /// system, meaning its runoff doesn't affect any other continents in any
    /// way.
    fn sim_continent_runoff(&mut self) {
        trace!("Simulating runoff for continent {}", self.id);
        self.calc_runoff_patterns();

        // Set initial runoff based on rainfall
        // We include sanity checks here to make sure we don't introduce any
        // new runoff after initialization
        self.initialize_runoff();
        let total_initial = self.total_runoff();

        // Push runoff downhill. In most cases this will eliminate a lot of
        // runoff from the system by pushing into the ocean
        self.push_downhill();
        // Sanity check
        let total_after_push = self.total_runoff();
        assert!(
            // Use a small delta to account for floating point error
            total_after_push <= total_initial + Meter3(1.0e-6),
            "Total runoff increased after pushing: continent={}, {} => {}",
            self.id,
            total_initial,
            total_after_push
        );

        // Simulate water backflowing. This can also eliminate more runoff if
        // backflow causes basins to overflow into the ocean
        self.simulate_backflow();
        // Sanity check
        let total_after_backflow = self.total_runoff();
        assert!(
            // Use a small delta to account for floating point error
            total_after_backflow <= total_after_push + Meter3(1.0e-6),
            "Total runoff increased after backflow: continent={} {} => {}",
            self.id,
            total_after_push,
            total_after_backflow
        );
    }

    /// Get the total amount of runoff on this continent. We use this for sanity
    /// checks, to make sure we're not accidentally introducing runoff during
    /// the simulation.
    fn total_runoff(&self) -> Meter3 {
        self.tiles.values().map(|tile| tile.runoff()).sum()
    }

    /// For each tile, calculate its runoff pattern. This pattern makes it easy
    /// to push runoff around later. Every tile in the continent will get a
    /// pattern, so the length of the output will match the length of the
    /// input. The output will be a map with all the same tiles as the
    /// input, with each tile paired to its runoff pattern.
    ///
    /// **This will reorder the input!** The continent needs to be sorted by
    /// ascending elevation to calculate runoff patterns.
    fn calc_runoff_patterns(&mut self) {
        // Build a map of runoff patterns for each tile. IMPORTANT: this map has
        // the same ordering as self.tiles, which we will use in the next step
        // to zip them together.
        //
        // This has to be done in two steps because borrow cking (we have to
        // reference multiple tiles at once during the first step).
        let mut runoff_patterns = HexPointIndexMap::default();
        for source_tile in self.tiles.values() {
            // For each neighbor of this tile, determine how much water it gets.
            // This is a list of (direction,elevation_diff) pairs
            let mut recipients: Vec<(HexDirection, Meter)> = Vec::new();
            for dir in HexDirection::iter() {
                let adj_pos = source_tile.position() + dir.to_vector();
                let adj_elev = match self.tiles.get(&adj_pos) {
                    // Adjacent tile isn't part of this continent, so assume
                    // it's ocean
                    None => World::SEA_LEVEL,
                    Some(adj_tile) => adj_tile.elevation(),
                };
                let elev_diff = source_tile.elevation() - adj_elev;
                // If neighbor is lower, we'll send runoff there. If not, then
                // ignore it
                if elev_diff > Meter(0.0) {
                    recipients.push((dir, elev_diff))
                }
            }

            // Distribute the water to our neighbors. Each neighbor gets an
            // amount proportional to the elevation different
            // between us and them. I.e. steeper slopes get more
            // water.

            let total_elev_diff: Meter =
                recipients.iter().map(|(_, elev_diff)| *elev_diff).sum();

            // For each adjacent lower tile, mark it as an exit in the pattern
            let mut runoff_pattern = RunoffPattern::new(source_tile.position());
            for (dir, elev_diff) in recipients {
                let adj_pos = source_tile.position() + dir.to_vector();
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

        // Join each runoff pattern into its corresponding tile
        for (tile, (_, runoff_pattern)) in
            self.tiles.values_mut().zip(runoff_patterns)
        {
            assert!(
                tile.position() == runoff_pattern.position(),
                "tile/runoff pattern position mismatch. \
                tile: {:?} \nrunoff pattern: {:?}",
                tile,
                runoff_pattern
            );
            tile.set_runoff_pattern(runoff_pattern);
        }
    }

    /// Generate an initial runoff level for every tile in a continent.
    fn initialize_runoff(&mut self) {
        // Set initial runoff for each tile
        for tile in self.tiles.values_mut() {
            // Set initial runoff level
            tile.set_runoff(tile.rainfall());
        }
    }

    /// Push all runoff on the continent downhill, so that it all ends up in
    /// just two places: terminal tiles and the ocean. Must runoff will end
    /// up in the ocean, but basins inside the continent will collect runoff
    /// at the terminals. Each basin will push all its runoff to only the single
    /// terminal tile though, so after this we will still have to simulate
    /// "backflow" to form lakes.
    ///
    /// This function tracks TWO stats: egress (for each tile, how much runoff
    /// exited in each direction?) and collected runoff (how much runoff remains
    /// on this tile after the downhill flow?).
    #[allow(clippy::needless_collect)]
    fn push_downhill(&mut self) {
        // Starting at the highest tile, we push the runoff from each tile down
        // to its lower neighbors. At each step, we track the egress in each
        // direction from the donor tile, and the ingress in the appropriate
        // direction for each donee tile.

        // Have to copy this into a vec to get around borrow checking
        // Clippy thinks we don't need this but we do, hence the #[allow] above
        // (If you don't believe me, try it yourself)
        let positions: Vec<_> = self.tiles.keys().copied().collect();

        // We have to iterate by index so that we can grab multiple mutable
        // tile refs in each iteration. The index lets us be more granular with
        // lifetimes, and doesn't affect time complexity.
        // .rev() is very important! we want to start at the highest tile
        for (i, source_pos) in positions.into_iter().enumerate().rev() {
            let (_, source_tile) = unwrap!(
                self.tiles.get_index_mut(i),
                "no tile for index {} in continent {}",
                i,
                self.id
            );
            let distribution = source_tile.distribute_runoff();
            // source_tile gets dropped now, which lets us grab a mutable ref
            // to each adjacent tile

            for (dir, amt) in distribution {
                let adj_pos = source_pos + dir.to_vector();
                // If the adjacent tile is in our continent, add our runoff to
                // if. If not, then it must be ocean so the runoff gets deleted
                if let Some(adj_tile) = self.tiles.get_mut(&adj_pos) {
                    adj_tile.add_runoff(amt, dir.opposite());
                }
            }
        }
    }

    /// Simulate "backflow", which is when runoff that has collected on a
    /// terminal tile spreads out to its neighbors. In some cases, the
    /// runoff on the terminal can be neatly distributed in its area, but in
    /// some cases it will overflow the terminal's basin, and some of it
    /// will end up flowing over into the ocean. We also need to handle
    /// cases where two terminal basins join to form a larger lake, or
    /// when one basin overflows into another but they DON'T join.
    fn simulate_backflow(&mut self) {
        // Initialize a basin for each terminal
        let mut basins = Basins::new(&mut self.tiles);

        // For each basin, attempt to spread out to its constituents. If one
        // basin overflows into another, then the recipient basin will be
        // re-queued. We'll continue until all runoff is settled. This will
        // eventually converge because we have logic to prevent cyclic overflow.
        let mut basin_queue: VecDeque<HexPoint> = basins.keys().collect();
        while let Some(basin_key) = basin_queue.pop_front() {
            let basin = basins.get_mut(basin_key);
            // Spread out water out as far as possible
            let overflow_distribution = self.grow_basin(basin);

            // If this basin overflowed into other(s), then do some processing
            // for each one
            for (overflow_dest, overflow_vol) in overflow_distribution {
                // If the overflow destination is a terminal tile (as opposed to
                // ocean), then push the overflow runoff into that basin
                if let RunoffDestination::Terminal(other_basin_key) =
                    overflow_dest
                {
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
                        let other_basin = basins.get_mut(other_basin_key);
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
                let tile = unwrap!(
                    self.tiles.get_mut(pos),
                    "unknown tile {} in continent {}",
                    pos,
                    self.id
                );
                let runoff_height = runoff_elev - tile.elevation();
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
    /// target(s) that it's overflowed to. If we didn't overflow, then it will
    /// be empty.
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

        let initial_basin_total = basin.runoff();

        // Each iteration of this loop will add a tile to the cluster, EXCEPT
        // for the last iteration. So for n iterations, we add n-1 tiles. This
        // loop will ALWAYS run at least once. In order for it not to, we'd have
        // to have a tile that (1) is a terminal and (2) has no land neighbors,
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
                candidate_tile.runoff() == Meter3(0.0),
                "encountered candidate tile with non-zero runoff {:?}",
                candidate_tile
            );

            // If the candidate is higher than our current water level, then
            // we can't reach it so the runoff stops spreading.
            let candidate_elev = candidate_tile.elevation();
            if candidate_elev >= basin.runoff_elevation() {
                break;
            }

            // If the candidate drains, totally or partially, somewhere else
            // (another basin(s) and/or the ocean), then we need to overflow
            // into those other target(s) before we can grow this basin anymore.
            // You can think of this as the water level of the basin rising up
            // until it starts to leak out at the lowest opening.
            let overflow_vol: Meter3 = (basin.runoff_elevation()
                - candidate_elev)
                * Tile::AREA
                * (basin.tiles().tiles().len() as f64);
            let overflow_distribution = basin.distribute_elsewhere(
                candidate_tile.runoff_pattern(),
                overflow_vol,
            );
            if !overflow_distribution.is_empty() {
                // Make sure the amount of runoff in the basin at the beginning
                // matches the amount at the end plus the amount we've removed
                // as overflow
                assert_approx_eq!(
                    initial_basin_total.0,
                    (basin.runoff() + overflow_vol).0
                );

                return overflow_distribution;
            }

            basin.add_tile(candidate_tile); // Initiation!
        }

        HashMap::default() // This won't allocate for an empty map
    }
}
