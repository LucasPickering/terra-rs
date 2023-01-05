mod generate;
pub mod hex;
pub mod tile;

use crate::{
    timed,
    util::{
        range::NumRange,
        unit::{Meter, Meter3},
    },
    world::{
        generate::WorldBuilder,
        hex::{TileDirection, TilePointMap},
        tile::Tile,
    },
    WorldConfig,
};
use anyhow::Context;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use validator::Validate;
#[cfg(feature = "js")]
use wasm_bindgen::prelude::*;

/// A fully generated world. Contains a collection of tiles as well the
/// configuration that was used to generate this world.
///
/// ## Serialization
/// Worlds can be serialized and deserialized through multiple formats: JSON and
/// binary.
///
/// ### JSON Format
/// The JSON format is fairly self-explanatory. It's intended to be as
/// consistent as possible, so all fields and values use snake casing. Some
/// external apps that consume this format may not support dynamic JSON objects,
/// so serialization avoids that by using either static sets of fields where
/// possible, and arrays of values rather than keyed objects in other cases.
///
/// ### Binary Format
/// Worlds can be saved and exported in a binary format via [World::to_bin] and
/// reloaded via [World::from_bin]. Currently the binary format is
/// [CBOR](https://cbor.io/), but that is subject to change so beware of that if
/// you write other programs that load the format.
#[cfg_attr(feature = "js", wasm_bindgen)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    /// The config used to generate this world. World generation is
    /// deterministic based on world config, and once the world has been
    /// generated, the config can never change.
    config: WorldConfig,

    /// The tiles that make up this world, keyed by their position.
    // Serialize as a vec because tile points can't be keys
    #[serde(with = "crate::util::serde_tile_point_map_to_vec")]
    tiles: TilePointMap<Tile>,
}

// Non-Wasm API
impl World {
    /// All tiles above this elevation are guaranteed to be non-ocean. All tiles
    /// at OR below _could_ be ocean, but the actual chance depends upon the
    /// ocean generation logic.
    pub const SEA_LEVEL: Meter = Meter(0.0);
    /// The range of possible elevation values. We guarantee that every tile's
    /// elevation will be in this range (inclusive on both ends).
    pub const ELEVATION_RANGE: NumRange<Meter, f64> =
        NumRange::new(Meter(-100.0), Meter(100.0));
    /// A **soft** range that defines possible rainfall values. Soft means that
    /// values can be outside this range! Specifically, they can be above the
    /// max (you can't have negative rainfall). There is no hard limit on how
    /// much rainfall a tile receives, so this range just defines "reasonable"
    /// values. We use this range to map to humidity.
    pub const RAINFALL_SOFT_RANGE: NumRange<Meter3, f64> =
        NumRange::new(Meter3(0.0), Meter3(5.0));

    /// Get a reference to the config that defines this world.
    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    /// Get a reference to the map of tiles that make up this world.
    pub fn tiles(&self) -> &TilePointMap<Tile> {
        &self.tiles
    }

    /// Get the owned tile map for this world
    pub fn into_tiles(self) -> TilePointMap<Tile> {
        self.tiles
    }

    /// Generate a new world with the given config. This operation could take
    /// several seconds, depending on the world size and complexity. Returns
    /// an error if the given config is invalid. Panics only in the case of
    /// internal bugs in the generation algorithm. Please report any panics
    /// on the git repo.
    pub fn generate(config: WorldConfig) -> anyhow::Result<Self> {
        info!("Generating world with config {:#?}", config);

        config.validate().context("invalid config")?;

        let tiles = timed!(
            "World generation",
            log::Level::Info,
            WorldBuilder::new(&config).generate_world()
        );

        Ok(Self { config, tiles })
    }

    /// Deserialize a world from JSON. A world can be serialized into JSON with
    /// [World::to_json]. Will fail if the input is malformed.
    #[cfg(feature = "json")]
    pub fn from_json(&self, json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).context("error deserializing world")
    }

    /// Deserialize a world from binary format. A world can be serialized into
    /// binary with [World::to_bin]. See the struct-level [World] documentation
    /// for a description of the binary format. Will fail if the input is
    /// malformed.
    #[cfg(feature = "bin")]
    pub fn from_bin(read: impl std::io::Read) -> anyhow::Result<Self> {
        serde_cbor::from_reader(read).context("error deserializing world")
    }
}

// Wasm-friendly API
#[cfg_attr(feature = "js", wasm_bindgen)]
impl World {
    /// Serializes this world into JSON. This is a recoverable format, which can
    /// be loaded back into a [World] with [World::from_json].
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> String {
        // Panic here indicates an internal bug in the data format
        serde_json::to_string(self).expect("error serializing world")
    }

    /// Serializes this world into a binary format. This is a recoverable
    /// format, which can be loaded back into a [World] with [World::from_bin].
    /// See the struct-level [World] documentation for a description of the
    /// binary format.
    #[cfg(feature = "bin")]
    pub fn to_bin(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Panic here indicates an internal bug in the data format
        serde_cbor::to_writer(&mut buffer, self)
            .expect("error serializing world");
        buffer
    }
}

/// High-level categories for biomes: land or water?
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BiomeType {
    Water,
    Land,
}

/// A biome is a large-scale classification of tile environment. Every tile can
/// be assigned a single biome based on its characteristics.
///
/// https://en.wikipedia.org/wiki/Biome
#[cfg_attr(feature = "js", wasm_bindgen)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Biome {
    // Water
    Ocean,
    Coast,

    // Land
    Snow,
    Desert,
    Alpine,
    Jungle,
    Forest,
    Plains,
}

impl Biome {
    /// Get this biome's high-level category
    pub fn biome_type(self) -> BiomeType {
        match self {
            Self::Ocean | Self::Coast => BiomeType::Water,
            Self::Snow
            | Self::Desert
            | Self::Alpine
            | Self::Jungle
            | Self::Forest
            | Self::Plains => BiomeType::Land,
        }
    }
}

/// A geographic feature is some feature that can appear on a tile. A tile can
/// have zero or more features (unlike biomes, where each tile gets exactly
/// one). Some feature combinations may be invalid (e.g. lake+beach) but that
/// isn't codified in the type system. Try not to mess it up.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeoFeature {
    /// Lakes are generated based on where water runoff collects. A lake takes
    /// up an entire tile. [See here](https://en.wikipedia.org/wiki/Lake) for
    /// more info.
    Lake,

    /// A river entering a tile from a specific direction. A tile can have
    /// multiple river entrances, but each one must have a unique direction and
    /// none of them can have the same direction as a river exit. These are
    /// generated based on runoff ingress measurements.
    RiverEntrance {
        direction: TileDirection,
        volume: Meter3,
    },
    /// A river exiting a tile from in a specific direction. A tile can have
    /// multiple river exits, but each one must have a unique direction and
    /// none of them can have the same direction as a river entrance. These
    /// are generated based on runoff egress measurements.
    RiverExit {
        direction: TileDirection,
        volume: Meter3,
    },
}
