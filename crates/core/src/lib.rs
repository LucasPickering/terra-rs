//! Terra is a hex tile-based terrain generation system. This crate contains
//! all the core generation logic. Presentation layers are implemented
//! elsewhere. See [terra.lucaspickering.me](https://terra.lucaspickering.me)
//! for a 3D rendering of what these worlds might look like.
//!
//! ```
//! use terra::{WorldConfig, World};
//!
//! let config = WorldConfig::default();
//! let world = World::generate(config).unwrap();
//! println!("{}", world.tiles().len());
//! // From here you can display/use the world however you like.
//! // See other methods on the World struct for possible output formats.
//! ```
//!
//! See [WorldConfig] for details on how the world generation can be customized.
//!
//! ## Features
//!
//! - `bin` - Import/export worlds to/from binary format
//!   ([World::from_bin]/[World::to_bin])
//! - `stl` - Render worlds in 3D STL format ([World::to_stl])
//! - `svg` - Render worlds in 2D SVG format ([World::to_svg])

#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_fn_trait_bound)]

mod config;
mod render;
mod util;
mod world;

pub use crate::{
    config::{
        GeoFeatureConfig, NoiseFnConfig, NoiseFnType, RainfallConfig,
        WorldConfig,
    },
    render::{config::RenderConfig, Color3, Point2, TileLens, WorldRenderer},
    util::{Meter, Meter2, Meter3, NumRange, RangeValue},
    world::{
        hex::{
            HasHexPosition, HexAxialDirection, HexAxis, HexDirection,
            HexDirectionMap, HexPoint, HexPointMap,
        },
        tile::Tile,
        Biome, BiomeType, GeoFeature, World,
    },
};
pub use anyhow;
pub use validator;
