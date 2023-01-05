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
//! - `bevy` - Enable Bevy ECS traits on types from this crate
//! - `bin` - Import/export worlds to/from binary format
//!   ([World::from_bin]/[World::to_bin])
//! - `json` - Import/export worlds to/from JSON format
//!   ([World::from_json]/[World::to_json])
//! - `stl` - Render worlds in 3D STL format ([World::to_stl])
//! - `svg` - Render worlds in 2D SVG format ([World::to_svg])
//! - `js` - Enable Wasm entrypoint for direct access from JS

#![feature(const_fn_floating_point_arithmetic)]

// TODO rework these to have one level of module depth

mod config;
#[cfg(feature = "js")]
pub mod js;
mod render;
mod util;
mod world;

pub use crate::{
    config::{
        ElevationConfig, GeoFeatureConfig, NoiseFnConfig, NoiseFnType,
        RainfallConfig, Seed, WorldConfig,
    },
    render::{config::RenderConfig, unit::Point2, TileLens, WorldRenderer},
    util::{
        range::{NumRange, RangeValue},
        unit::{Meter, Meter2, Meter3},
    },
    world::{
        hex::{
            HasHexPosition, HexAxialDirection, HexAxis, HexDirection,
            TileDirection, TileDirectionMap, TilePoint, TilePointMap,
            VertexDirection,
        },
        tile::Tile,
        Biome, BiomeType, GeoFeature, World,
    },
};
pub use anyhow;
pub use validator;
