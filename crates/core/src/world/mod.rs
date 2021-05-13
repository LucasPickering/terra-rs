mod generate;
pub mod hex;

use crate::{
    timed,
    util::{Color3, Meter, Meter2, Meter3, NumRange},
    world::{
        generate::WorldBuilder,
        hex::{
            HasHexPosition, HexDirection, HexDirectionMap, HexPoint,
            HexPointMap,
        },
    },
    WorldConfig,
};
use anyhow::Context;
use log::info;
use serde::{Deserialize, Serialize};
use std::io::Read;
use strum::EnumString;
use validator::Validate;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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

/// A fully generated world. Contains a collection of tiles as well the
/// configuration that was used to generate this world.
///
///
/// ## Binary Format
/// Worlds can be saved and exported in a binary format via [World::to_bin] and
/// reloaded via [World::from_bin]. Currently the binary format is just msgpack,
/// but that is subject to change so beware of that if you write other programs
/// that load the format.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    config: WorldConfig,
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

    /// Generate a new world with the given config. This operation could take
    /// several seconds, depending on the world size and complexity.
    pub fn generate(config: WorldConfig) -> anyhow::Result<Self> {
        info!("Generating world with config {:#?}", config);

        config.validate().context("invalid config")?;

        let tiles = timed!(
            "World generation",
            log::Level::Info,
            WorldBuilder::new(config)
                .generate_world()
                .context("error during world validation")?
        );

        Ok(Self { config, tiles })
    }

    /// Deserialize a world from JSON. A world can be serialized into JSON with
    /// [World::to_json]. Will fail if the input is malformed.
    pub fn from_json(&self, json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).context("error deserializing world")
    }

    /// Serializes this world into JSON. This is a recoverable format, which can
    /// be loaded back into a [World] with [World::from_json].  A failure
    /// here indicates a bug in Terra that prevents serialization.
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string(self).context("error serializing world")
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
    /// binary format. A failure here indicates a bug in Terra that prevents
    /// serialization.
    #[cfg(feature = "bin")]
    pub fn to_bin(&self) -> anyhow::Result<Vec<u8>> {
        rmp_serde::to_vec_named(self).context("error serializing world")
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
        util::svg::world_to_svg(self, lens, show_features).to_string()
    }

    /// Render this world into an STL model. Return value is the STL binary
    /// data. Returns an error if serialization fails, which indicates a bug
    /// in terra or stl_io.
    #[cfg(feature = "stl")]
    pub fn to_stl(&self) -> anyhow::Result<Vec<u8>> {
        use crate::util;
        let mesh = util::stl::world_to_stl(self);
        let mut buffer = Vec::<u8>::new();
        stl_io::write_stl(&mut buffer, mesh.iter())
            .context("error serializing STL")?;
        Ok(buffer)
    }
}

/// A world is comprised of tiles. Each tile is a hexagon (in 2D renderings) or
/// a hexagonal prism (in 3D renderings). In the case of the prism, a tile's
/// height is determined by its elevation. Tiles **cannot** be stacked.
///
/// A tile has certain geographic properties, and when we combine a bunch of
/// tiles together, we get terrain.
///
/// Tiles can't be constructed directly, they can only be made by the world
/// generation process. See [World::generate]. They also can't be modified after
/// world generation.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    /// The location of this tile in the world. See [HexPoint] for a
    /// description of the coordinate system. Every tile in the world has a
    /// unique position.
    position: HexPoint,
    /// The elevation of this tile, relative to sea level.
    elevation: Meter,
    /// Amount of rain that fell on this tile during rain simulation.
    rainfall: Meter3,
    /// Amount of runoff water that remains on the tile after runoff
    /// simulation.
    runoff: Meter3,
    /// The net amount of runoff gained/lost for this tile in each direction.
    /// Positive values indicate ingress (i.e. runoff came in from that
    /// direction) and negative values indicate egress (i.e. runoff left in
    /// that direction). The value should be positive if the neighbor in that
    /// direction is a higher elevation, and negative if it is lower.
    ///
    /// It's _possible_ for this map to not have 6 entries, if a tile doesn't
    /// have 6 neighbors or if it had 0 ingress/egress with one of the
    /// neighbors, but in most scenarios there will be an entry for all 6
    /// neighbors.
    runoff_traversed: HexDirectionMap<Meter3>,
    /// The biome for this tile. Every tile exists in a single biome, which
    /// describes its climate characteristics. See [Biome] for more info.
    biome: Biome,
    /// All geographic features on this tile. A geographic feature describes
    /// some unique formation that can appear on a tile. No two features in
    /// this vec can be identical.
    features: Vec<GeoFeature>,
}

// Non-Wasm API
impl Tile {
    /// The top surface area of a single tile, in abstract units! Note that this
    /// math doesn't line up with [Tile::VERTEX_RADIUS] or the other rendering
    /// constants, i.e. if you were to calculate an
    pub const AREA: Meter2 = Meter2(1.0);

    // Rendering constants below
    /// Distance between the center of a tile and one of its 6 vertices, in
    /// **2D space**. This is also the length of one side of the tile.
    ///
    /// ## Rendering Constant Caveat
    /// This value is **not** consistent with the abstract units of [Meter]/
    /// [Meter2]/[Meter3]. There is some artistic license employed during
    /// rendering. See [Point2] for a description of 2D space.
    pub const VERTEX_RADIUS: f64 = 1.0;
    /// Distance between the center of a tile and the midpoint of one of its
    /// sides, in **2D space**. See [Tile::VERTEX_RADIUS] for the rendering
    /// constant caveat.
    pub const SIDE_RADIUS: f64 = Self::VERTEX_RADIUS * 0.8660254; // sqrt(3)/2
    /// Distance between the bottom side and top side of a tile, in **2D
    /// space**. See [Tile::VERTEX_RADIUS] for the rendering constant caveat.
    pub const HEIGHT: f64 = Self::SIDE_RADIUS * 2.0;

    /// Get a list of geographic features that appear on this tile. See
    /// [GeoFeature] for more info.
    ///
    /// **Note**: NOT available to WebAssembly. `wasm-bindgen` doesn't support
    /// complex enums, so we can't pass [GeoFeature] across the Wasm boundary.
    pub fn features(&self) -> &[GeoFeature] {
        self.features.as_slice()
    }
}

// Wasm-friendly API
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Tile {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn pos(&self) -> HexPoint {
        self.position
    }

    /// Return the elevation of the top of this tile, relative to sea level.
    /// This value is guaranteed to be in the range [Self::ELEVATION_RANGE].
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn elevation(&self) -> Meter {
        self.elevation
    }

    /// Tile elevation, but mapped to a zero-based range so the value is
    /// guaranteed to be non-negative. This makes it safe to use for vertical
    /// scaling during rendering.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn height(&self) -> Meter {
        World::ELEVATION_RANGE
            .map_to(&World::ELEVATION_RANGE.zeroed(), self.elevation)
    }

    /// Total amount of water that fell on this tile during rain simulation.
    /// This value is guaranteed to be non-negative, but has no hard maximum.
    /// If you need to map a rainfall value to some bounded range, you can use
    /// [Self::RAINFALL_SOFT_RANGE] for a soft maximum.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn rainfall(&self) -> Meter3 {
        self.rainfall
    }

    /// A normalized (meaning [0,1]) proxy for rainfall. Since rainfall is an
    /// unbounded range, we define an arbitrary soft maximum for it, and
    /// anything at/above that max will map to 1.0 humidity. Anything between
    /// the min (0) and the soft max will map proportionally to [0,1] to
    /// determine humidity.
    ///
    /// This function will **always** return a value in [0,1].
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn humidity(&self) -> f64 {
        World::RAINFALL_SOFT_RANGE
            .value(self.rainfall)
            .clamp()
            .convert::<f64>()
            .normalize()
            .inner()
    }

    /// The amount of water runoff that collected on this tile. This is the
    /// amount of runoff **currently** on the tile after runoff simulation,
    /// **not** the amount of total runoff that passed over the tile.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn runoff(&self) -> Meter3 {
        self.runoff
    }

    /// Get the tile's biome. Every tile will have exactly on biome assigned.
    /// See [Biome] for more info.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn biome(&self) -> Biome {
        self.biome
    }

    /// Compute the color of a tile based on the lens being viewed. The lens
    /// controls what data the color is derived from.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn color(&self, lens: TileLens) -> Color3 {
        match lens {
            TileLens::Surface => {
                if self.features.contains(&GeoFeature::Lake) {
                    Ok(Color3::new_int(72, 192, 240))
                } else {
                    Ok(self.biome.color())
                }
            }
            TileLens::Biome => Ok(self.biome.color()),
            TileLens::Elevation => {
                let normal_elev =
                    World::ELEVATION_RANGE.normalize(self.elevation()).0 as f32;
                // 0 -> white
                // 1 -> red
                Color3::new(1.0, 1.0 - normal_elev, 1.0 - normal_elev)
            }
            TileLens::Humidity => {
                let humidity = self.humidity() as f32;
                // 0 -> white
                // 1 -> green
                Color3::new(1.0 - humidity, 1.0, 1.0 - humidity)
            }
            TileLens::Runoff => {
                // This coloring is based on two aspects: runoff (how much water
                // collected on the tile) AND runoff egress (how much water
                // flowed over the tile without staying there). Runoff controls
                // blue, runoff egress controls green.
                if self.biome.biome_type() == BiomeType::Water {
                    Color3::new(0.5, 0.5, 0.5)
                } else {
                    let normal_runoff = NumRange::new(Meter3(0.0), Meter3(5.0))
                        .value(self.runoff)
                        .normalize()
                        // Runoff doesn't have a fixed range so we have to clamp
                        // this to make sure we don't overflow the color value
                        .clamp()
                        .convert::<f64>()
                        .inner() as f32;

                    let runoff_egress: Meter3 = self
                        .runoff_traversed
                        .values()
                        .copied()
                        // Only include egress, ignore ingress
                        .filter(|v| *v < Meter3(0.0))
                        .map(|Meter3(v)| Meter3(v.abs()))
                        .sum();
                    let normal_runoff_egress =
                        NumRange::new(Meter3(0.0), Meter3(1000.0))
                            .value(runoff_egress)
                            .normalize()
                            .clamp()
                            .convert::<f64>()
                            .inner() as f32;

                    // (0,0) -> black
                    // (1,0) -> blue
                    // (0,1) -> green
                    // (1,1) -> cyan
                    Color3::new(0.0, normal_runoff_egress, normal_runoff)
                }
            }
        }
        // this is hard to remove because we can't pass an anyhow result to wasm
        .unwrap()
    }
}

impl HasHexPosition for Tile {
    fn position(&self) -> HexPoint {
        self.position
    }
}

/// A definition of what data is used to compute a tile's color.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TileLens {
    /// Color is based on a combination of biome and geographic features.
    Surface,
    /// Color is based solely on the tile's biome. Each biome has a unique
    /// static color.
    Biome,
    /// Color is a gradient based on elevation.
    Elevation,
    /// Color is a gradient based on humidity.
    Humidity,
    /// Color is based on a combination of runoff and total runoff egress.
    Runoff,
}
