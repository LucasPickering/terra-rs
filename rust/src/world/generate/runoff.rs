use crate::world::{
    generate::Generate,
    hex::{
        Cluster, HasHexPosition, HexDirection, HexPoint, HexPointMap, WorldMap,
    },
    tile::TileBuilder,
    World, WorldConfig,
};
use derive_more::Display;
use log::error;
use rand::Rng;
use std::{cmp::Ordering, collections::HashMap, default::Default, iter};
use strum::IntoEnumIterator;

/// Each tile gets some amount of initial runoff based on its humidity. This is
/// the conversion factor.
const HUMIDITY_TO_RUNOFF_SCALE: f64 = 0.1;

/// Simulate water runoff. This applies some amount of rainfall to each tile,
/// then simulates the water flowing downhill. This doesn't actually do
/// anything with the runoff values though, it just sets them. A separate
/// generator is responsible for turning the runoff values into
/// lakes/rivers/etc.
///
/// This needs to run AFTER ocean generation!
#[derive(Clone, Debug, Display)]
#[display(fmt = "Runoff Generator")]
pub struct RunoffGenerator;

impl Generate for RunoffGenerator {
    fn generate(
        &self,
        _: &WorldConfig,
        _: &mut impl Rng,
        tiles: &mut WorldMap<TileBuilder>,
    ) {
        let continents = tiles.clusters_predicate(|tile| !tile.is_water());
        // Hypothetically we could run these simulations in parallel since each
        // continent is independent, but skipping that for now cause Wasm.
        for continent in continents {
            sim_continent_runoff(continent.tiles);
        }
    }
}

/// Compare two tiles by their elevation
fn cmp_elev(a: &TileBuilder, b: &TileBuilder) -> Ordering {
    a.elevation()
        .unwrap()
        .partial_cmp(&b.elevation().unwrap())
        .unwrap()
}

/// Simulate runoff for a single continent. Each continent is an independent
/// system, meaning its runoff doesn't affect any other continents in any way.
fn sim_continent_runoff(mut continent: HexPointMap<&mut TileBuilder>) {
    gen_initial_runoff(&mut continent);
    let mut continent: HexPointMap<(&mut TileBuilder, RunoffPattern)> =
        calc_runoff_patterns(continent);
    push_downhill(&mut continent);
    sim_backflow(&mut continent);
}

/// Generate an initial runoff level for every tile in a continent.
fn gen_initial_runoff(continent: &mut HexPointMap<&mut TileBuilder>) {
    // Set initial runoff for each tile
    for tile in continent.values_mut() {
        // Set initial runoff level
        tile.set_runoff(tile.humidity().unwrap() * HUMIDITY_TO_RUNOFF_SCALE);
    }
}

/// For each tile, calculate its runoff pattern. This pattern makes it easy to
/// push runoff around later. Every tile in the continent will get a pattern,
/// so the length of the output will match the length of the input. The output
/// will be a map with all the same tiles as the input, with each tile paired
/// to its runoff pattern.
///
/// **This will reorder the input!** The continent needs to be sorted by
/// ascending elevation to calculate runoff patterns.
fn calc_runoff_patterns(
    mut continent: HexPointMap<&mut TileBuilder>,
) -> HexPointMap<(&mut TileBuilder, RunoffPattern)> {
    // Sort tiles by ascending elevation. This is very important! Runoff
    // patterns have to be generated low->high so the patterns of their lower
    // neighbors. Once we have a pattern for each tile, we can easily
    // calculate where water ends up for each tile.
    continent.sort_by(|_, a, _, b| cmp_elev(a, b));

    // Build a map of runoff patterns for each tile. IMPORTANT: this map has
    // the same ordering as the continent, which allows us to do index lookups
    // instead of key lookups later. gotta go fast
    let mut runoff_patterns: HexPointMap<RunoffPattern> =
        HexPointMap::with_capacity_and_hasher(
            continent.len(),
            Default::default(),
        );
    for source_tile in continent.values() {
        // For each neighbor of this tile, determine how much water it gets.
        // This is a map of direction:elevation_diff
        let recipients: Vec<(HexDirection, f64)> = HexDirection::iter()
            .filter_map(|dir| {
                let adj_pos = source_tile.position() + dir.offset();
                let adj_elev = match continent.get(&adj_pos) {
                    // Adjacent tile isn't part of this continent, so assume
                    // it's ocean. (Right now that isn't entirely true because
                    // the map gets cut off, but at some point we'll ensure that
                    // the map always has a water boundary)
                    None => World::SEA_LEVEL,
                    Some(adj_tile) => adj_tile.elevation().unwrap(),
                };
                let elev_diff = source_tile.elevation().unwrap() - adj_elev;
                if elev_diff > 0.0 {
                    // Neighbor is lower, we'll send runoff there
                    Some((dir, elev_diff))
                } else {
                    // Neighbor is higher, ignore it
                    None
                }
            })
            .collect();

        // Distribute the water to our neighbors. Each neighbor gets an amount
        // proportional to the elevation different between us and them. I.e.
        // steeper slopes get more water.

        let total_elev_diff: f64 =
            recipients.iter().map(|(_, elev_diff)| elev_diff).sum();

        // For each adjacent lower tile, mark it as an exit in the pattern
        let mut runoff_pattern = RunoffPattern::new(source_tile.position());
        for (dir, elev_diff) in recipients {
            let adj_pos = source_tile.position() + dir.offset();
            runoff_pattern.add_exit(
                dir,
                // This is why the tiles have to be ascending by elevation,
                // because we back-reference the lower tiles
                runoff_patterns.get(&adj_pos),
                elev_diff / total_elev_diff,
            );
        }
        runoff_patterns.insert(source_tile.position(), runoff_pattern);
    }

    // Zip the two maps together. We have to do this into a separate step
    // because borrow checking
    continent
        .into_iter()
        .zip(runoff_patterns.into_iter())
        .map(|((pos, tile), (_, pattern))| (pos, (tile, pattern)))
        .collect()
}

/// Push all runoff on the continent downhill, so that it all ends up in just
/// two places: terminal tiles and the ocean. Must runoff will end up in the
/// ocean, but holes/dips inside the continent will collect runoff at the
/// terminals. Each dip will only have a single terminal though, so after this
/// step we will still need to simulate "backflow".
fn push_downhill(
    continent: &mut HexPointMap<(&mut TileBuilder, RunoffPattern)>,
) {
    // Now that we have our runoff patterns, we can figure out how much water
    // ends up going to each terminal. We have to do this in two steps because
    // borrow checking.
    let mut terminal_runoffs: HexPointMap<f64> = HexPointMap::default();
    for (source_tile, source_pattern) in continent.values_mut() {
        let to_distribute = source_tile.clear_runoff();
        // Add the appropriate amount of water to each terminal. For most tiles,
        // the terminals' factors don't add up to 1, so some or all of the water
        // gets deleted from the system (i.e. flows to the ocean).
        for (term_pos, factor) in source_pattern.terminals().iter() {
            *terminal_runoffs.entry(*term_pos).or_insert(0.0) +=
                to_distribute * factor;
        }
    }
    for (pos, runoff) in terminal_runoffs {
        continent.get_mut(&pos).unwrap().0.add_runoff(runoff);
    }
}

/// Simulate "backflow", which is when runoff that has collected on a terminal
/// tile spreads out to its neighbors. In some cases, the runoff on the terminal
/// can be neatly distributed in its area, but in some cases it will overflow
/// the terminal's hole/dip, and some of it will end up flowing over into the
/// ocean. We also need to handle cases where two terminal clusters join to form
/// a larger lake, or when one cluster overflows into another but they DON'T
/// join.
fn sim_backflow(
    continent: &mut HexPointMap<(&mut TileBuilder, RunoffPattern)>,
) {
    // For each terminal, map it to its constituents (all the other tiles that
    // it will spread to)
    let mut terminal_holes: HexPointMap<Cluster<()>> = continent
        .iter()
        .filter(|(_, (_, pattern))| pattern.is_terminal())
        .map(|(pos, _)| {
            let init_map: HexPointMap<()> = iter::once((*pos, ())).collect();
            (*pos, Cluster::new(init_map))
        })
        .collect();

    // For each terminal, we'll try to spread its water around
    for (terminal_pos, hole_cluster) in terminal_holes.iter_mut() {
        // Ok so here's the deal: We have a single terminal tile with a bunch of
        // runoff on it, and we need to distribute it around. The general
        // approach is:
        // 1. Find the lowest neighbor to the hole
        // 2. See if we have enough runoff to overflow onto that neighbor
        //   a. If so, then overflow onto it and repeat from step 1
        //   b. If not, then our cluster is complete

        let (terminal_tile, _) = continent.get(terminal_pos).unwrap();
        // "Runoff elevation" is elevation+runoff for any tile, i.e. the
        // elevation of the surface of the water. Since water is a
        // liquid[citation needed], it will spread evenly across the cluster
        // which means all tiles in the cluster will have the same runoff
        // elevation -- that's what this value is.
        let mut current_runoff_elev =
            terminal_tile.elevation().unwrap() + terminal_tile.runoff();

        // Each iteration of this loop will add a tile to the cluster, EXCEPT
        // for the last iteration. So for n iterations, we add n-1 tiles. This
        // loop will ALWAYS run at least once. In order for it not to, we'd have
        // to have a tile that is (1) a terminal and (2) has no land neighbors,
        // which doesn't make any sense.
        while let Some((candidate_tile, candidate_pattern)) = hole_cluster
            .adjacents()
            .iter()
            .filter_map(|pos| continent.get(pos))
            .min_by(|a, b| cmp_elev(a.0, b.0))
        {
            if candidate_pattern.terminals.len() > 1 {
                // If we reach a tile that spreads into another cluster, just
                // punt for now. This case is hard to handle
                // TODO figure it out
                error!("Tried to add tile with multiple terminals, fix this ya doof. source={}", terminal_pos);
                break;
            }

            // Just a sanity check. We expect every tile that's not a terminal
            // to have no runoff on it. Once cluster-joining is working, this
            // check probably won't make sense anymore.
            assert!(candidate_tile.runoff() == 0.0);

            // If the candidate is higher than our current water level, then
            // we can't reach it so the runoff stops spreading.
            let candidate_elev = candidate_tile.elevation().unwrap();
            if candidate_elev >= current_runoff_elev {
                break;
            }

            // If we can overflow onto the candidate, but that tile drains
            // partially (or fully) to the ocean, then any runoff we put on it
            // will just drain away. That means our cluster's water level can
            // only get as high as the candidate, so that's our upper limit.
            if candidate_pattern.drains_to_ocean() {
                current_runoff_elev = candidate_elev;
                break;
            }

            // Candidate has been elected! Welcome to the club.
            hole_cluster.insert(candidate_tile.position(), ()); // Initiation!

            // This math is neat. When we induct the candidate, we're not adding
            // any more runoff to the system, but there's one more tile worth
            // of space to fill, so the water level across the cluster has to
            // drop. First, calculate how much empty space we've just added to
            // the system.
            let added_volume = current_runoff_elev - candidate_elev;
            // Now spread that volume shortfall across the whole cluster
            current_runoff_elev -=
                added_volume / (hole_cluster.tiles.len() as f64);
            // (tbh I'm not 100% sure this is correct but maybe like 98%)
        }

        // Now we know which tiles our runoff spreads to, so we can distribute
        for pos in hole_cluster.tiles.keys() {
            let (tile, _) = continent.get_mut(pos).unwrap();
            tile.set_runoff(current_runoff_elev - tile.elevation().unwrap());
        }
    }
}

/// A runoff pattern is essentially a way of memoizing parts of the runoff
/// generation process. When we calculate runoff, we start at the lowest tiles
/// and for each one, figure out how its runoff will flow to its neighbors,
/// based on elevation differences. Obviously, shit flows downhill. Since the
/// elevation is static, we can determine the runoff pattern for a tile with
/// abstract/normalized values, then use those patterns to distribute the actual
/// runoff later.
///
/// A runoff pattern for a tile has two components: the exit pattern, and
/// the terminal pattern. The tile associated with a runoff pattern is called
/// the source tile.
///
/// The exit pattern for a tile holds information about every step that
/// runoff takes after leaving the source. If you put 1.0 water on the source
/// and start runoff, the traversal pattern tells you exactly which tiles that
/// water will run over, and exactly how much water will go over each tile.
///
/// A runoff pattern does not hold any state, meaning it does not maintain any
/// information about how much water any tile holds, how much water has
/// traversed the tile, etc. The pattern only maintains proportional information
/// of how water should move from/through the source.
#[derive(Clone, Debug)]
struct RunoffPattern {
    /// Position of the source tile that this pattern is built for.
    position: HexPoint,

    /// The neighbors of this tile, and how much water each one gets from this
    /// tile. This map will only include entries for tiles that actually get
    /// some water, and all the values should sum to 1 (unless it's empty). If
    /// this map is empty, the tile is a terminal.
    exits: HashMap<HexDirection, f64>,

    /// A terminal is a tile with no exits. The terminal map shows where runoff
    /// from this will end up. Each key is a terminal tile and the value is a
    /// fraction [0, 1] denoting how much of the source's runoff should end
    /// up on that terminal. The values in this map should sum to 1, unless
    /// some or all of the water gets dumped into the ocean. The difference
    /// between their sum and 1 is the portion of runoff that gets expelled
    /// to the ocean. **If a tile has no terminals, then all of its runoff
    /// ends up in the ocean and it is dubbed a "sink".** The vast majority of
    /// land tiles will end up being sinks.
    terminals: HexPointMap<f64>,
}

impl RunoffPattern {
    fn new(position: HexPoint) -> Self {
        Self {
            position,
            exits: HashMap::new(),
            terminals: HexPointMap::default(),
        }
    }

    /// Is this tile a terminal? A terminal is a tile with no exits.
    fn is_terminal(&self) -> bool {
        self.exits.is_empty()
    }

    fn terminals(&self) -> &HexPointMap<f64> {
        &self.terminals
    }

    /// Check if **some** (or all) of the water from this tile drains to the
    /// ocean.
    fn drains_to_ocean(&self) -> bool {
        self.terminals.values().sum::<f64>() + 0.000001 < 1.0
    }

    /// Add a new exit to this pattern. The exit has a specific direction and
    /// factor. All of a the factors for all of a tile's exits should sum to 1.
    fn add_exit(
        &mut self,
        dir: HexDirection,
        other_pattern: Option<&RunoffPattern>,
        factor: f64,
    ) {
        self.exits.insert(dir, factor);

        // If the other tile has a runoff pattern, then use it to figure out
        // where our terminals are. If not, then that means it's ocean.
        if let Some(other_pattern) = other_pattern {
            if other_pattern.is_terminal() {
                // This exit is a terminal itself, so add/update it to our map
                *self.terminals.entry(other_pattern.position).or_insert(0.0) +=
                    factor;
            } else {
                // This exit is NOT a terminal, so add all its terminals to us
                for (p, f) in &other_pattern.terminals {
                    // We want to add the other tile's terminal, but with one
                    // more degree of separation, like so: us->other->term
                    // f is the amt of water that goes other->term, so we want
                    // to scale that by the us->other factor, to get us->term
                    let term_factor = f * factor;
                    // If we're already sending some runoff to this terminal,
                    // make sure we update that value instead of overwriting
                    *self.terminals.entry(*p).or_insert(0.0) += term_factor;
                }
            }
        }
    }
}
