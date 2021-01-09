use crate::{
    world::hex::{HasHexPosition, HexDirection, HexPoint, HexPointMap},
    Meter3,
};
use fnv::FnvBuildHasher;
use std::collections::HashMap;

/// Runoff can terminate at either the ocean or at specific tile.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RunoffDestination {
    Ocean,
    Terminal(HexPoint),
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
pub struct RunoffPattern {
    /// Position of the source tile that this pattern is built for.
    position: HexPoint,

    /// The neighbors of this tile, and how much water each one gets from this
    /// tile. This map will only include entries for tiles that actually get
    /// some water, and all the values should sum to 1 (unless it's empty). If
    /// this map is empty, the tile is a terminal.
    exits: HashMap<HexDirection, f64, FnvBuildHasher>,

    /// A map that tracks every tile that runoff passes over after leaving this
    /// tile. This **doesn't** track where the runoff _ends up_ (that's what
    /// `terminals` is for). These numbers represent the fraction of runoff
    /// from this origin tile that passes over each descendent. E.g. if
    /// this tile starts with 1.0 m³ of runoff, and the map looks like
    /// this:
    ///
    /// ```text
    /// {
    ///     (1, 1, -2) => 0.3,
    ///     (-1, 1, 0) => 0.7,
    ///     (-1, 2, -1) => 0.4,
    ///     (-1, 0, 1) => 0.3,
    /// }
    /// ```
    ///
    /// Then 30% of our runoff passes over (1, 1, -2), 70% over (-1, 1, 0), and
    /// so on. Notice that the values **do not add up to 1.0**. The same blob
    /// of runoff will run over many tiles, so 1.0 m³ of runoff can account for
    /// much more than 1.0 m³ of traversal.
    traversals: HexPointMap<f64>,

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
            traversals: HexPointMap::default(),
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

    /// Distribute the given runoff quantity to each of this pattern's exits.
    /// The returned map determines how much runoff each exit direction
    /// receives. The values of the returned map will always sum to 1,
    /// **unless** this tile is a terminal. In that case, it has no exits,
    /// so the returned map will be empty.
    pub fn distribute_exits(
        &self,
        runoff: Meter3,
    ) -> HashMap<HexDirection, Meter3, FnvBuildHasher> {
        self.exits
            .iter()
            .map(|(dir, f)| (*dir, runoff * f))
            .collect()
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
                    .entry(RunoffDestination::Terminal(other_pattern.position))
                    .or_default() += factor;
            } else {
                // This exit is NOT a terminal, so add all its terminals to us
                for (d, f) in &other_pattern.terminals {
                    // We want to add the other tile's terminal, but with one
                    // more degree of separation, like so: us->other->term
                    // f is the amt of water that goes other->term, so we want
                    // to scale that by the us->other factor, to get us->term
                    let term_factor = f * factor;
                    // If we're already sending some runoff to this terminal,
                    // make sure we update that value instead of overwriting
                    *self.terminals.entry(*d).or_default() += term_factor;
                }
            }
        } else {
            // The exit is an ocean tile, so denote that with the None key
            *self.terminals.entry(RunoffDestination::Ocean).or_default() +=
                factor;
        }
    }
}

impl HasHexPosition for RunoffPattern {
    fn position(&self) -> HexPoint {
        self.position
    }
}
