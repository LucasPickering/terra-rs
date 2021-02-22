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
//! ```
//!
//! See [WorldConfig] for details on how the world generation can be customized.

#![feature(cmp_min_max_by)]
#![feature(const_fn)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(map_into_keys_values)]

mod config;
mod util;
mod world;

pub use crate::{
    config::{GeoFeatureConfig, NoiseFnConfig, RainfallConfig, WorldConfig},
    util::{Color3, Meter, Meter2, Meter3, NumRange, Point2, RangeValue},
    world::{
        hex::{
            HasHexPosition, HexAxialDirection, HexAxis, HexDirection,
            HexDirectionMap, HexPoint, HexPointMap,
        },
        Biome, BiomeType, GeoFeature, Tile, TileLens, World,
    },
};
pub use validator;
