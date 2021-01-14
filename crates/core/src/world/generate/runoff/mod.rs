mod basin;
mod pattern;

use crate::{
    unwrap_or_bail,
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
use anyhow::{ensure, Context};
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
    fn generate(&self, world: &mut WorldBuilder) -> anyhow::Result<()> {
        let continents = Cluster::predicate(&mut world.tiles, |tile| {
            Ok(!tile.is_water_biome())
        })?;
        // Hypothetically we could run these simulations in parallel since each
        // continent is independent, but skipping that for now cause Wasm.
        for continent in continents {
            let mut continent = Continent::new(continent.into_tiles())?;
            continent.sim_continent_runoff()?;
        }

        Ok(())
    }
}

/// Compare two tiles by their elevation
fn cmp_elev(a: &TileBuilder, b: &TileBuilder) -> Ordering {
    util::cmp_unwrap(&a.elevation().unwrap(), &b.elevation().unwrap())
}

/// A cluster of land tiles. One tile cannot belong to more than one continent.
struct Continent<'a> {
    /// A point that unique identifies this continent. This point should be any
    /// tile within the continent, since each tile cannot belong to any other
    /// continent. Exactly which tile it is isn't important, since it isn't
    /// used for calculations, just as a unique ID.
    id: HexPoint,
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
    fn new(
        mut tiles: HexPointIndexMap<&'a mut TileBuilder>,
    ) -> anyhow::Result<Self> {
        let (&id, _) = unwrap_or_bail!(
            tiles.first(),
            "cannot initialize empty continent",
        );
        let runoff_patterns = Self::calc_runoff_patterns(&mut tiles)?;
        Ok(Self {
            id,
            tiles,
            runoff_patterns,
        })
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
    ) -> anyhow::Result<HexPointIndexMap<RunoffPattern>> {
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
            // This is a list of (direction,elevation_diff) pairs
            let mut recipients: Vec<(HexDirection, Meter)> = Vec::new();
            for dir in HexDirection::iter() {
                let adj_pos = source_tile.position() + dir.to_vector();
                let adj_elev = match tiles.get(&adj_pos) {
                    // Adjacent tile isn't part of this continent, so assume
                    // it's ocean
                    None => World::SEA_LEVEL,
                    Some(adj_tile) => adj_tile.elevation()?,
                };
                let elev_diff = source_tile.elevation()? - adj_elev;
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

        Ok(runoff_patterns)
    }

    /// Simulate runoff for a single continent. Each continent is an independent
    /// system, meaning its runoff doesn't affect any other continents in any
    /// way.
    fn sim_continent_runoff(&mut self) -> anyhow::Result<()> {
        trace!("Simulating runoff for continent {}", self.id);
        self.initialize_runoff()?;
        self.push_downhill()?;
        self.sim_backflow()?;
        Ok(())
    }

    /// Generate an initial runoff level for every tile in a continent.
    fn initialize_runoff(&mut self) -> anyhow::Result<()> {
        // Set initial runoff for each tile
        for tile in self.tiles.values_mut() {
            // Set initial runoff level
            tile.set_runoff(tile.rainfall()?)?;
        }
        Ok(())
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
    fn push_downhill(&mut self) -> anyhow::Result<()> {
        // Important notes for this func:
        // Ingress - total runoff that has ENTERED a tile
        // Egress - total runoff that has EXITED a tile
        // Runoff - runoff that remains on a tile at the end of this func
        //      i.e. ingress-egress
        // For a given tile, ingress and egress can only ever increase!

        // Step 1 - Initialize a map that tracks ingress for each tile. We
        // initialize it by MOVING all the runoff from each tile into
        // this map. That means during the next step, this map is the only
        // source of truth for how much runoff is in the system!
        //
        // This is slightly hacky since tracking ingress actually means we
        // duplicate runoff at each step, but it works out because in
        // the end the only runoff remaining in the system is what collects in
        // the terminals, and for a terminal tile, runoff=ingress (because
        // egress is always 0).
        let mut total_ingress: HexPointMap<Meter3> = self
            .tiles
            .values_mut()
            .map(|tile| Ok((tile.position(), tile.clear_runoff()?)))
            .collect::<anyhow::Result<_>>()?;

        // Step 2 - Starting at the top, calculate how much runoff each tile
        // pushes to its neighbors.
        for (source_tile, source_pattern) in self
            .tiles
            .values_mut()
            .zip(self.runoff_patterns.values())
            // Very important - tiles are sorted low-to-high, so flip it
            .rev()
        {
            let source_pos = source_tile.position();

            // Sanity check to make sure our maps line up
            ensure!(
                source_pos == source_pattern.position(),
                "source tile has position {}, \
                    but corresponding runoff pattern has position {} \
                    (in continent {})",
                source_pos,
                source_pattern.position(),
                self.id
            );

            // This value will be whatever we started with (based on rainfall)
            // plus whatever total our neighbors have pushed down to us
            let source_ingress = *unwrap_or_bail!(
                total_ingress.get(&source_pos),
                "no runoff ingress for {}",
                source_pos
            );

            // Whatever ingress enters this tile will get egressed to our lower
            // neighbors, which have been pre-calculated in the runoff pattern.
            for (dir, adj_ingress) in
                source_pattern.distribute_exits(source_ingress)
            {
                let adj_pos = source_pos + dir.to_vector();
                *total_ingress.entry(adj_pos).or_default() += adj_ingress;
            }
        }

        // Step 3 - Now that we know how much runoff has entered each tile,
        // then for each tile we can calculate the egress (for non-terminal
        // tiles) or how much collected on the tile (for terminals)
        for (pos, tile) in self.tiles.iter_mut() {
            // Defensive coding! thanks Bryan
            let runoff_pattern = unwrap_or_bail!(
                self.runoff_patterns.get_mut(pos),
                "no runoff pattern for tile {}",
                pos
            );
            let tile_ingress = *unwrap_or_bail!(
                total_ingress.get(&pos),
                "no runoff ingress value for tile {}",
                pos
            );

            let runoff_egress = runoff_pattern.distribute_exits(tile_ingress);
            let is_terminal = runoff_egress.is_empty();
            tile.set_runoff_egress(runoff_egress);

            // If this tile has no egress, that means it's a terminal. In that
            // case, whatever ingress we get remains here, so add it back to
            // the tile as collected runoff
            if is_terminal {
                tile.add_runoff(tile_ingress)?;
            }
        }

        Ok(())
    }

    /// Simulate "backflow", which is when runoff that has collected on a
    /// terminal tile spreads out to its neighbors. In some cases, the
    /// runoff on the terminal can be neatly distributed in its area, but in
    /// some cases it will overflow the terminal's basin, and some of it
    /// will end up flowing over into the ocean. We also need to handle
    /// cases where two terminal clusters join to form a larger lake, or
    /// when one cluster overflows into another but they DON'T join.
    fn sim_backflow(&mut self) -> anyhow::Result<()> {
        // For each terminal, map it to its constituents (all the other tiles
        // that it will spread to)
        let mut basins = Basins::new(&mut self.tiles)?;

        let mut basin_queue: VecDeque<HexPoint> = basins.keys().collect();
        while let Some(basin_key) = basin_queue.pop_front() {
            let basin = basins.get_mut(basin_key).context("queued basin")?;
            let overflow_distribution = self.grow_basin(basin)?;

            // If this basin overflowed into other(s), then do some processing
            // for each one
            for (overflow_dest, overflow_vol) in overflow_distribution {
                // If the overflow destination is a terminal tile (as opposed to
                // ocean), then push the overflow runoff into that basin
                if let RunoffDestination::Terminal(other_basin_key) =
                    overflow_dest
                {
                    if basins
                        .has_previously_overflowed(other_basin_key, basin_key)?
                    {
                        // This other basin has already donated to us. Since
                        // we've overflowed in both directions now, the two need
                        // to be joined
                        let joined_basin = basins.join(
                            basin_key,
                            other_basin_key,
                            overflow_vol,
                        )?;

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
                        let other_basin = basins
                            .get_mut(other_basin_key)
                            .context("joined basin")?;
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
                let tile = unwrap_or_bail!(
                    self.tiles.get_mut(pos),
                    "unknown tile {} in continent {}",
                    pos,
                    self.id
                );
                let runoff_height = runoff_elev - tile.elevation()?;
                // Convert Meter -> Meter3
                tile.set_runoff(runoff_height * Tile::AREA)?;
            }
        }

        Ok(())
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
    ) -> anyhow::Result<HashMap<RunoffDestination, Meter3, FnvBuildHasher>>
    {
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
            ensure!(
                candidate_tile.runoff()? == Meter3(0.0),
                "encountered candidate tile with non-zero runoff {:?}",
                candidate_tile
            );

            // If the candidate is higher than our current water level, then
            // we can't reach it so the runoff stops spreading.
            let candidate_elev = candidate_tile.elevation()?;
            if candidate_elev >= basin.runoff_elevation() {
                break;
            }

            // If the candidate drains, totally or partially, somewhere else
            // (another basin(s) and/or the ocean), then we need to overflow
            // into those other target(s) before we can grow this basin anymore.
            // You can think of this as the water level of the basin rising up
            // until it starts to leak out at the lowest opening.
            let candidate_pattern = unwrap_or_bail!(
                self.runoff_patterns.get(&candidate_tile.position()),
                "no runoff pattern found for {} in continent {}",
                candidate_tile.position(),
                self.id
            );
            let overflow_vol: Meter3 = (basin.runoff_elevation()
                - candidate_elev)
                * Tile::AREA
                * (basin.tiles().tiles().len() as f64);
            let overflow_distribution =
                basin.distribute_elsewhere(candidate_pattern, overflow_vol);
            if !overflow_distribution.is_empty() {
                return Ok(overflow_distribution);
            }

            basin.add_tile(candidate_tile)?; // Initiation!
        }

        Ok(HashMap::default()) // This won't allocate for an empty map
    }
}
