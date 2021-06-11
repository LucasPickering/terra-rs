//! This module holds basic types and data structures related to hexagon grids.
//!
//! ## Coordinate Systems
//!
//! Terra uses two different coordinate systems:
//!
//! ### World Coordinates
//!
//! World coordinates (AKA hex coordinates) define space within the hexagon-tile
//! based Terra world. The system we use is an extension of the [cube coordinate
//! system defined by Amit Patel](https://www.redblobgames.com/grids/hexagons/#coordinates-cube).
//!
//! #### Basic Description
//!
//! The description in the link above is much better than anything I can write
//! here, but here's a TL;DR just for the hell of it:
//!
//! Each coordinate has three components (`x`, `y`, and `z`). The most common
//! object that is referenced in the coordinate system is tiles. **For any tile
//! coordinate, all three components will be integers and `x + y + z = 0`.**
//! Even though hexagon tiles are mapped out in two dimensions (the vertical
//! dimension is used strictly for elevation, i.e. tiles can't stack), we use
//! three-dimensional coordinates to make math around hexagonal grids much
//! simpler. The hex coordinate system is defined by a three-dimensional step
//! function projected onto the plane `x + y + z = 0`.
//!
//! #### Our Extension
//!
//! While the original cube coordinate system only uses integer coordinates
//! on the plane `x + y + z = 0`, our extensions allow coordinates to reference
//! any point in the cube step function (TODO step function). I.e., they can
//! reference any point within a hexagonal grid, **not just tile centers**. That
//! means tile vertices, midpoints of tile sides (the boundaries between tiles),
//! or any other point in the hex grid.
//!
//! Visit [hex-demo.lucaspickering.me](https://hex-demo.lucaspickering.me/) for
//! an interactive demo of the cube coordinate system. This helps visualize how
//! points can simultaneously refer to points on the cube step function and
//! within the hex grid. Visually, the step function looks like a cascade of
//! cubes, and each cube corresponds to a single tile and has eight vertices. Of
//! those eight, the furthest from the plane `x + y + z = 0` (which is the
//! vertex not visible in the demo) never gets used, leaving us with seven. The
//! vertex that falls on that plane is the center of the tile, and the other six
//! are the vertices of that tile's hexagon.
//!
//! **Note:** This system makes a very key assumption: tiles are contiguous,
//! meaning there are no gaps between tiles. This means that two adjacent tiles
//! will share exactly one side and two vertices. This is important because it
//! affects both the world and screen spaces, and restricts how worlds can be
//! rendered. This is a reasonable assumption though, because rendering
//! multi-tile features like lakes or rivers would be weird with visual gaps
//! between the tiles. The benefit of this assumption is that it makes the world
//! space math more self-consistent.
//!
//! ### Screen Coordinates
//!
//! While world coordinates are the most commonly used coordinate system in
//! Terra, there is a second system: screen coordinates. While world coordinates
//! define space in the world for hte purposes of calculating distance,
//! direction, etc., screen coordinates are used strictly for rendering a world
//! into some sort of visual output. This coordinate system is much easier to
//! conceptualize: it depicts a Terra world from the top-down (birds eye view).
//! In the following diagram, `o` (the origin) represents the tile `(0, 0, 0)`.
//!
//! +-------------------+
//! |        +y         |
//! |         ^         |
//! |         |         |
//! | -x <----o----> +x |
//! |         |         |
//! |         v         |
//! |        -y         |
//! +-------------------+
//!
//! In a rendering, the three-dimensional world coordinates are converted to 2D
//! because in conventional Cartesian space, a hex grid only has two dimensions,
//! and the third is provided by tile elevation.
//!
//! You can imagine this coordinate system as the coordinates you would use if
//! you were to render a Terra world with the following camera settings:
//!
//! - Position: `(0, 0, 10)` (or any position directly above the origin)
//! - Orientation: Looking straight down
//! - Mode: Orthographic
//!
//! #### Usage
//!
//! These coordinates are only used during rendering calculations. This doesn't
//! necessarily mean rendering directly onto a screen though, and also doesn't
//! mean they can only be used for 2D output formats! 3D outputs can still be
//! achieved by leveraging each tile's elevation as well.
//!
//! See the [crate::render] module for utilities around rendering tiles and
//! worlds.
//!
//! #### Calculation
//!
//! Converting from world coordinates to screen coordinates involves these
//! steps:
//!
//! 1. Project the hex point onto the plane `x + y + z = 0`
//! 2. Rotate the plane to equal `z = 0`
//!   1. Rotate 45 degress around the `z` axis
//!   2. Rotate 45 defress around the `x` axis
//!
//! Use [HexPoint::to_screen_space] to perform this conversion for any world
//! point.

mod data_structure;
mod unit;

pub use self::{data_structure::*, unit::*};
