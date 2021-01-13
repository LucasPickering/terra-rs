//! Terra is a hex tile-based terrain generation system. This crate contains
//! all the core generation logic. Presentation layers are implemented
//! elsewhere. See [terra.lucaspickering.me](https://terra.lucaspickering.me)
//! for a 3D rendering of what these worlds might look like.
//!
//! ```
//! use terra::{WorldConfig, World};
//!
//! let config = WorldConfig::default();
//! let world = World::generate(config);
//! println!("{}", world.tiles().len());
//! // From here you can display/use the world however you like.
//! ```
//!
//! See [WorldConfig] for details on how the world generation can be customized.

#![feature(cmp_min_max_by)]
#![feature(const_fn)]

mod config;
mod util;
mod world;

pub use crate::{
    config::{NoiseFnConfig, WorldConfig},
    util::{Meter, Meter2, Meter3, NumRange, RangeValue},
    world::{hex::HasHexPosition, Biome, BiomeType, Tile, TileLens, World},
};
