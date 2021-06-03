mod generate;
pub mod hex;
pub mod tile;

use crate::{
    timed,
    util::{Color3, Meter, Meter3, NumRange, TileLens},
    world::{
        generate::WorldBuilder,
        hex::{HexDirection, HexPointMap},
        tile::Tile,
    },
    WorldConfig,
};
use anyhow::Context;
use log::info;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, io::Read};
use validator::Validate;
#[cfg(target_arch = "wasm32")]
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
/// reloaded via [World::from_bin]. Currently the binary format is just msgpack,
/// but that is subject to change so beware of that if you write other programs
/// that load the format.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    /// The config used to generate this world. World generation is
    /// deterministic based on world config, and once the world has been
    /// generated, the config can never change.
    config: WorldConfig,

    /// The tiles that make up this world, keyed by their position.
    // Serialize as a vec because hex points can't be keys
    #[serde(with = "crate::util::hex_point_map_to_vec_serde")]
    tiles: HexPointMap<Tile>,
}

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
    pub fn tiles(&self) -> &HexPointMap<Tile> {
        &self.tiles
    }

    /// Get the height that a tile's geometry should have. This will convert
    /// the tile's elevation to a zero-based scale, then multiplicatively scale
    /// it based on the pre-configured Y scale of the world. See
    /// [RenderConfig::y_scale] for more info on what exactly the vertical scale
    /// means.
    pub fn tile_render_height(&self, tile: &Tile) -> f64 {
        // Map elevation to a zero-based scale
        let zeroed_elevation = World::ELEVATION_RANGE
            .map_to(&World::ELEVATION_RANGE.zeroed(), tile.elevation);
        // Multiply by render scale
        zeroed_elevation.0 * self.config.render.y_scale
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
            WorldBuilder::new(config).generate_world()
        );

        Ok(Self { config, tiles })
    }

    /// Deserialize a world from JSON. A world can be serialized into JSON with
    /// [World::to_json]. Will fail if the input is malformed.
    pub fn from_json(&self, json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).context("error deserializing world")
    }

    /// Serializes this world into JSON. This is a recoverable format, which can
    /// be loaded back into a [World] with [World::from_json].
    pub fn to_json(&self) -> String {
        // Panic here indicates an internal bug in the data format
        serde_json::to_string(self).expect("error serializing world")
    }

    /// Deserialize a world from binary format. A world can be serialized into
    /// binary with [World::to_bin]. See the struct-level [World] documentation
    /// for a description of the binary format. Will fail if the input is
    /// malformed.
    #[cfg(feature = "bin")]
    pub fn from_bin(read: impl Read) -> anyhow::Result<Self> {
        rmp_serde::from_read(read).context("error deserializing world")
    }

    /// Serializes this world into a binary format. This is a recoverable
    /// format, which can be loaded back into a [World] with [World::from_bin].
    /// See the struct-level [World] documentation for a description of the
    /// binary format.
    #[cfg(feature = "bin")]
    pub fn to_bin(&self) -> Vec<u8> {
        // Panic here indicates an internal bug in the data format
        rmp_serde::to_vec_named(self).expect("error serializing world")
    }

    /// Render this world as a 2D SVG, from a top-down perspective. Returns the
    /// SVG in a string.
    ///
    /// ## Params
    /// - `lens` - The [crate::TileLens] to use when determining each tile's
    ///   color
    /// - `show_features` - Should geographic features (lakes, rivers, etc.) be
    ///   rendered? See [crate::GeoFeature] for a full list
    #[cfg(feature = "svg")]
    pub fn to_svg(&self, lens: TileLens, show_features: bool) -> String {
        use crate::util;
        let svg = util::svg::world_to_svg(self, lens, show_features);
        svg.to_string()
    }

    /// Render this world into an STL model. Return value is the STL binary
    /// data. Returns an error if serialization fails, which indicates a bug
    /// in terra or stl_io.
    #[cfg(feature = "stl")]
    pub fn to_stl(&self) -> Vec<u8> {
        use crate::util;
        let mesh = util::stl::world_to_stl(self);
        let mut buffer = Vec::<u8>::new();
        // Panic here indicates a bug in our STL mesh format
        stl_io::write_stl(&mut buffer, mesh.iter())
            .expect("error serializing STL");
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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

    /// Get a pretty color unique to this biome
    pub fn color(self) -> Color3 {
        match self {
            Self::Ocean => Color3::new_int(20, 77, 163),
            Self::Coast => Color3::new_int(32, 166, 178),

            Self::Snow => Color3::new_int(191, 191, 191),
            Self::Desert => Color3::new_int(214, 204, 107),
            Self::Alpine => Color3::new_int(99, 122, 99),
            Self::Jungle => Color3::new_int(43, 179, 31),
            Self::Forest => Color3::new_int(23, 122, 0),
            Self::Plains => Color3::new_int(173, 201, 115),
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
        direction: HexDirection,
        volume: Meter3,
    },
    /// A river exiting a tile from in a specific direction. A tile can have
    /// multiple river exits, but each one must have a unique direction and
    /// none of them can have the same direction as a river entrance. These
    /// are generated based on runoff egress measurements.
    RiverExit {
        direction: HexDirection,
        volume: Meter3,
    },
}
