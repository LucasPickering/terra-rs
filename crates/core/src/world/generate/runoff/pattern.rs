use crate::{
    world::hex::{HasHexPosition, TileDirection, TileDirectionMap, TilePoint},
    Meter3,
};
use assert_approx_eq::assert_approx_eq;
use fnv::FnvBuildHasher;
use std::collections::HashMap;

/// Runoff can terminate at either the ocean or at specific tile.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RunoffDestination {
    Ocean,
    Terminal(TilePoint),
}

pub type RunoffDestinationMap<T> =
    HashMap<RunoffDestination, T, FnvBuildHasher>;

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
    position: TilePoint,

    /// The neighbors of this tile, and how much water each one gets from this
    /// tile. This map will only include entries for tiles that actually get
    /// some water, and all the values should sum to 1 (unless it's empty). If
    /// this map is empty, the tile is a terminal.
    exits: TileDirectionMap<f64>,

    /// A destination is a point that collects runoff. There are two types:
    /// - Ocean
    /// - Terminal tile, which is a tile with no exits, i.e. a tile whose
    /// neighbors all have a higher elevation than it
    ///
    /// This map shows where runoff from this pattern will end up. Each key is
    /// a destination and the value is a fraction [0, 1] denoting how much of
    /// the source's runoff should end up on that terminal. Any runoff that
    /// flows to the ocean will exit the continent's runoff system, i.e. the
    /// runoff gets deleted. The values in this map should **always** sum to
    /// 1, **unless** this tile is a terminal itself, in which it has no
    /// destinations and this map will be empty. **If a tile has no terminals,
    /// then all of its runoff ends up in the ocean and it is dubbed a
    /// "sink".** The vast majority of land tiles will end up being sinks.
    destinations: RunoffDestinationMap<f64>,
}

impl RunoffPattern {
    pub fn new(position: TilePoint) -> Self {
        Self {
            position,
            exits: HashMap::default(),
            destinations: HashMap::default(),
        }
    }

    /// Is this tile a terminal? A terminal is a tile with no exits.
    pub fn is_terminal(&self) -> bool {
        self.exits.is_empty()
    }

    /// Distribute the given runoff quantity to each of this pattern's exits.
    /// The returned map indicates how much runoff each exit direction
    /// receives. The values of the returned map will always sum to the input
    /// runoff amount, **unless** this tile is a terminal. In that case, it has
    /// no exits, so the returned map will be empty.
    pub fn distribute_to_exits(
        &self,
        runoff: Meter3,
    ) -> TileDirectionMap<Meter3> {
        self.exits
            .iter()
            .map(|(dir, f)| (*dir, runoff * f))
            .collect()
    }

    /// Filter out some terminals from this pattern's destinations, and scale
    /// the remaining destination factors so that they still sum to 1. This
    /// function answers the question "Where would the runoff go if it
    /// _couldn't_ flow to these particular tiles?"
    pub fn filter_destinations(
        &self,
        excluding: &[TilePoint],
    ) -> RunoffDestinationMap<f64> {
        // Remove any destinations that match the specified terminals
        let mut filtered_destinations: RunoffDestinationMap<f64> = self
            .destinations
            .iter()
            .filter_map(|(destination, fraction)| match destination {
                RunoffDestination::Terminal(term_pos)
                    if excluding.contains(term_pos) =>
                {
                    None
                }
                _ => Some((*destination, *fraction)),
            })
            .collect();

        // We need to scale up the remaining destinations so that they still sum
        // to 1. Since the old sum was 1, we can just divide each remaining
        // value by the new sum to get back to 1
        let filtered_sum: f64 = filtered_destinations.values().sum();
        for value in filtered_destinations.values_mut() {
            *value /= filtered_sum;
        }

        // Sanity check: Make sure the new distribution destinations add up to
        // 1.0. If we filtered the destinations down to empty though, then
        // obviously they can't add up to one, so skip that case
        if !filtered_destinations.is_empty() {
            assert_approx_eq!(filtered_destinations.values().sum::<f64>(), 1.0);
        }

        filtered_destinations
    }

    /// Add a new exit to this pattern. The exit has a specific direction and
    /// factor. All of the factors for all of a tile's exits should sum to 1.
    pub fn add_exit(
        &mut self,
        dir: TileDirection,
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
                    .destinations
                    .entry(RunoffDestination::Terminal(other_pattern.position))
                    .or_default() += factor;
            } else {
                // This exit is NOT a terminal, so add all its terminals to us
                for (d, f) in &other_pattern.destinations {
                    // We want to add the other tile's terminal, but with one
                    // more degree of separation, like so: us->other->term
                    // f is the amt of water that goes other->term, so we want
                    // to scale that by the us->other factor, to get us->term
                    let term_factor = f * factor;
                    // If we're already sending some runoff to this terminal,
                    // make sure we update that value instead of overwriting
                    *self.destinations.entry(*d).or_default() += term_factor;
                }
            }
        } else {
            // The exit is an ocean tile, so denote that with the None key
            *self
                .destinations
                .entry(RunoffDestination::Ocean)
                .or_default() += factor;
        }
    }
}

impl HasHexPosition for RunoffPattern {
    type Point = TilePoint;
    fn position(&self) -> TilePoint {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_destinations() {
        let mut pattern = RunoffPattern::new(TilePoint::new_xy(0, 0));
        pattern
            .destinations
            .insert(RunoffDestination::Terminal(TilePoint::new_xy(0, 1)), 0.2);
        pattern
            .destinations
            .insert(RunoffDestination::Terminal(TilePoint::new_xy(1, 0)), 0.2);
        pattern.destinations.insert(RunoffDestination::Ocean, 0.6);

        let output = pattern.filter_destinations(&[TilePoint::new_xy(1, 0)]);

        assert_eq!(output.len(), 2);
        // Each of these values should be scaled up to fill in the gap left by
        // the filtered terminal(s). In this case, each one gets divided by 0.8
        assert_approx_eq!(
            output
                .get(&RunoffDestination::Terminal(TilePoint::new_xy(0, 1)))
                .unwrap(),
            0.25
        );
        assert_approx_eq!(output.get(&RunoffDestination::Ocean).unwrap(), 0.75);
    }
}
