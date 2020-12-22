use crate::world::{
    generate::Generate,
    hex::{HasHexPosition, HexDirection, HexPoint, HexPointMap},
    tile::TileBuilder,
    World, WorldConfig,
};
use derive_more::Display;
use rand::Rng;
use std::{collections::HashMap, default::Default};
use strum::IntoEnumIterator;

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
        tiles: &mut HexPointMap<TileBuilder>,
    ) {
        let continents = tiles.clusters_predicate(|tile| !tile.is_water());
        for continent in continents {
            sim_continent_runoff(continent.0);
        }
    }
}

fn initial_runoff(_tile: &TileBuilder) -> f64 {
    0.1 // TODO make this dynamic based on humidity
}

fn sim_continent_runoff(mut continent: HexPointMap<&mut TileBuilder>) {
    // Set initial runoff for each tile
    for tile in continent.values_mut() {
        // Set initial runoff level
        tile.add_runoff(initial_runoff(tile));
    }

    // Sort tiles by ascending elevation. This is very important! Runoff
    // patterns have to be generated low->high so the patterns of their lower
    // neighbors. Once we have a pattern for each tile, we can easily
    // calculate where water ends up for each tile.
    continent.sort_by(|_, a, _, b| {
        a.elevation()
            .unwrap()
            .partial_cmp(&b.elevation().unwrap())
            .unwrap()
    });

    // Build a map of runoff patterns for each tile. IMPORTANT: this map has
    // the same ordering as the continent, which allows us to do index lookups
    // instead of key lookups later. gotta go fast
    let mut runoff_patterns: HexPointMap<RunoffPattern> = HexPointMap::new();
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
        let mut runoff_pattern = RunoffPattern::default();
        for (dir, elev_diff) in recipients {
            let adj_pos = source_tile.position() + dir.offset();
            runoff_pattern.add_exit(
                source_tile.position(),
                dir,
                // This is why the tiles have to be ascending by elevation,
                // because we back-reference the lower tiles
                runoff_patterns.get(&adj_pos),
                elev_diff / total_elev_diff,
            );
        }
        runoff_patterns.insert(source_tile.position(), runoff_pattern);
    }

    // Now that we have our runoff patterns, we can figure out how much water
    // ends up going to each terminal. We have to do this in two steps because
    // borrow checking.
    let mut terminal_runoffs: HexPointMap<f64> = HexPointMap::new();
    for (i, tile) in continent.values_mut().enumerate() {
        let to_distribute = tile.clear_runoff();
        // optimization here - look up by index
        let (_, source_pattern) = runoff_patterns.get_index(i).unwrap();
        // Add the appropriate amount of water to each terminal. For most tiles,
        // the terminals' factors don't add up to 1, so some or all of the water
        // gets deleted from the system (i.e. flows to the ocean).
        for (term_pos, factor) in source_pattern.terminals().iter() {
            *terminal_runoffs.entry(*term_pos).or_insert(0.0) +=
                to_distribute * factor;
        }
    }

    for (pos, runoff) in terminal_runoffs {
        continent.get_mut(&pos).unwrap().add_runoff(runoff);
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
#[derive(Clone, Debug, Default)]
pub struct RunoffPattern {
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
    /// Is this tile a terminal? A terminal is a tile with no exits.
    pub fn is_terminal(&self) -> bool {
        self.exits.is_empty()
    }

    pub fn terminals(&self) -> &HexPointMap<f64> {
        &self.terminals
    }

    /// TODO
    pub fn add_exit(
        &mut self,
        source_pos: HexPoint,
        dir: HexDirection,
        other_pattern: Option<&RunoffPattern>,
        factor: f64,
    ) {
        self.exits.insert(dir, factor);

        // If the other tile has a runoff pattern, then use it to figure out
        // where our terminals are. If not, then that means it's ocean.
        if let Some(other_pattern) = other_pattern {
            if other_pattern.is_terminal() {
                let other_pos = source_pos + dir.offset();
                // This exit is a terminal itself, so add it to our map
                self.terminals.insert(other_pos, factor);
            } else {
                // This exit is NOT a terminal, so add all its terminals to us
                self.terminals.extend(
                    other_pattern
                        .terminals
                        .iter()
                        .map(|(p, f)| (*p, f * factor)),
                );
            }
        }
    }
}
