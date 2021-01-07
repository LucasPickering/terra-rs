use crate::world::hex::{HexDirection, HexPoint};
use fnv::FnvBuildHasher;
use std::collections::HashMap;

/// Runoff can terminate at either the ocean (`None`) or a specific tile
/// (`Some`).
pub type RunoffDestination = Option<HexPoint>;

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
pub struct RunoffPattern {
    /// Position of the source tile that this pattern is built for.
    position: HexPoint,

    /// The neighbors of this tile, and how much water each one gets from this
    /// tile. This map will only include entries for tiles that actually get
    /// some water, and all the values should sum to 1 (unless it's empty). If
    /// this map is empty, the tile is a terminal.
    exits: HashMap<HexDirection, f64, FnvBuildHasher>,

    /// A terminal is a tile with no exits. The terminal map shows where runoff
    /// from this will end up. Each key is a terminal tile (or `None`) and the
    /// value is a fraction [0, 1] denoting how much of the source's runoff
    /// should end up on that terminal. If the key is `None`, that denotes
    /// the ocean, meaning that fraction of runoff exists the system. The
    /// values in this map should **always** sum to 1. **If a tile has no
    /// terminals, then all of its runoff ends up in the ocean and it is
    /// dubbed a "sink".** The vast majority of land tiles will end up
    /// being sinks.
    terminals: HashMap<RunoffDestination, f64, FnvBuildHasher>,
}

impl RunoffPattern {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            exits: HashMap::default(),
            terminals: HashMap::default(),
        }
    }

    pub fn terminals(
        &self,
    ) -> &HashMap<RunoffDestination, f64, FnvBuildHasher> {
        &self.terminals
    }

    /// Is this tile a terminal? A terminal is a tile with no exits.
    pub fn is_terminal(&self) -> bool {
        self.exits.is_empty()
    }

    /// Add a new exit to this pattern. The exit has a specific direction and
    /// factor. All of the factors for all of a tile's exits should sum to 1.
    pub fn add_exit(
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
                *self
                    .terminals
                    .entry(Some(other_pattern.position))
                    .or_insert(0.0) += factor;
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
        } else {
            // The exit is an ocean tile, so denote that with the None key
            *self.terminals.entry(None).or_insert(0.0) += factor;
        }
    }
}
