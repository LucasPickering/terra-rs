mod generate;
pub mod hex;

use std::collections::HashMap;

use crate::{
    timed,
    util::{Color3, Meter, Meter2, Meter3, NumRange},
    world::{
        generate::WorldBuilder,
        hex::{HasHexPosition, HexDirection, HexPoint, HexPointMap},
    },
    WorldConfig,
};
use anyhow::Context;
use fnv::FnvBuildHasher;
use log::{info, Level};
use serde::{Deserialize, Serialize};
use validator::Validate;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsCast};

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
    RiverEntrance(HexDirection),
    /// A river exiting a tile from in a specific direction. A tile can have
    /// multiple river exits, but each one must have a unique direction and
    /// none of them can have the same direction as a river entrance. These
    /// are generated based on runoff egress measurements.
    RiverExit(HexDirection),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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
            Level::Info,
            WorldBuilder::new(config)
                .generate_world()
                .context("error during world validation")?
        );

        Ok(Self { config, tiles })
    }
}

// Wasm-only functions
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl World {
    /// A type-hacked accessor to get all tiles in the world from Wasm. This
    /// typing can be cleaned up after https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen]
    pub fn wasm_tiles(self) -> TileArray {
        use js_sys::Array;

        self.tiles
            .into_values()
            .map(JsValue::from)
            .collect::<Array>()
            .unchecked_into()
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
    /// Amount of runoff that **exited** this tile in each direction. This map
    /// will only contain entries for non-zero egress directions, meaning
    /// any direction for which the adjacent tile is uphill (higher elevation)
    /// will *not* have an entry here.
    runoff_egress: HashMap<HexDirection, Meter3, FnvBuildHasher>,
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

                    let normal_runoff_egress =
                        NumRange::new(Meter3(0.0), Meter3(1000.0))
                            .value(self.runoff_egress.values().copied().sum())
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
#[derive(Copy, Clone, Debug)]
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

// Types that we can't natively return. These are assigned TS types, but
// these types aren't actually verified by the compiler. Be careful
// here!
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {

    /// Type hack needed until https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen(typescript_type = "Tile[]")]
    pub type TileArray;
}