mod generate;
pub mod hex;

use crate::{
    timed,
    util::{Color3, Meter, Meter2, Meter3, NumRange},
    world::{
        generate::WorldBuilder,
        hex::{HasHexPosition, HexPoint, WorldMap},
    },
    WorldConfig,
};
use js_sys::Array;
use log::info;
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BiomeType {
    Water,
    Land,
}

/// A biome is a large-scale classification of tile environment. Every tile can
/// be assigned a single biome based on its characteristics.
///
/// https://en.wikipedia.org/wiki/Biome
///
/// TODO separate the concept of "biome" from "feature"?
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Biome {
    // Water
    Ocean,
    Coast,
    Lake,

    // Land
    Snow,
    Desert,
    Alpine,
    Jungle,
    Forest,
    Plains,
}

impl Biome {
    pub fn biome_type(self) -> BiomeType {
        match self {
            Self::Ocean | Self::Coast | Self::Lake => BiomeType::Water,
            Self::Snow
            | Self::Desert
            | Self::Alpine
            | Self::Jungle
            | Self::Forest
            | Self::Plains => BiomeType::Land,
        }
    }

    pub fn color(self) -> Color3 {
        match self {
            Self::Ocean => Color3::new(0.08, 0.30, 0.64).unwrap(),
            Self::Coast => Color3::new_int(32, 166, 178),
            Self::Lake => Color3::new_int(72, 192, 240),

            Self::Snow => Color3::new(0.75, 0.75, 0.75).unwrap(),
            Self::Desert => Color3::new(0.84, 0.80, 0.42).unwrap(),
            Self::Alpine => Color3::new(0.39, 0.48, 0.37).unwrap(),
            Self::Jungle => Color3::new(0.17, 0.70, 0.12).unwrap(),
            Self::Forest => Color3::new(0.09, 0.48, 0.0).unwrap(),
            Self::Plains => Color3::new(0.68, 0.79, 0.45).unwrap(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    tiles: WorldMap<Tile>,
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

    pub fn tiles(&self) -> &WorldMap<Tile> {
        &self.tiles
    }

    pub fn generate(config: WorldConfig) -> Self {
        info!("Generating world");
        let tiles = timed!(
            "World generation",
            WorldBuilder::new(config).generate_world()
        );
        Self { config, tiles }
    }
}

#[wasm_bindgen]
impl World {
    /// A type-hacked accessor to get all tiles in the world for Wasm. This
    /// typing can be cleaned up after https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen]
    pub fn tiles_array(&self) -> TileArray {
        self.tiles
            .iter()
            .copied()
            .map(JsValue::from)
            .collect::<Array>()
            .unchecked_into()
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct Tile {
    position: HexPoint,
    elevation: Meter,
    /// Amount of rain that fell on this tile during rain simulation.
    rainfall: Meter3,
    /// Amount of runoff water that remains on the tile after runoff
    /// simulation.
    runoff: Meter3,
    biome: Biome,
}

impl Tile {
    /// The top surface area of a single tile.
    pub const AREA: Meter2 = Meter2(1.0);
}

#[wasm_bindgen]
impl Tile {
    #[wasm_bindgen(getter)]
    pub fn pos(&self) -> HexPoint {
        self.position
    }

    /// Return the elevation of the top of this tile, relative to sea level.
    /// This value is guaranteed to be in the range [Self::ELEVATION_RANGE].
    #[wasm_bindgen(getter)]
    pub fn elevation(&self) -> Meter {
        self.elevation
    }

    /// Tile elevation, but mapped to a zero-based range so the value is
    /// guaranteed to be non-negative. This makes it safe to use for vertical
    /// scaling during rendering.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> Meter {
        World::ELEVATION_RANGE
            .map_to(&World::ELEVATION_RANGE.zeroed(), self.elevation)
    }

    /// Total amount of water that fell on this tile during rain simulation.
    /// This value is guaranteed to be non-negative, but has no hard maximum.
    /// If you need to map a rainfall value to some bounded range, you can use
    /// [Self::RAINFALL_SOFT_RANGE] for a soft maximum.
    #[wasm_bindgen(getter)]
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
    #[wasm_bindgen(getter)]
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
    #[wasm_bindgen(getter)]
    pub fn runoff(&self) -> Meter3 {
        self.runoff
    }

    /// Get the tile's biome. Every tile will have exactly on biome assigned.
    /// See [Biome] for more info.
    #[wasm_bindgen(getter)]
    pub fn biome(&self) -> Biome {
        self.biome
    }

    /// Compute the color of a tile based on the lens being viewed. The lens
    /// controls what data the color is derived from.
    #[wasm_bindgen]
    pub fn color(&self, lens: TileLens) -> Color3 {
        match lens {
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
                let normal_runoff = NumRange::new(Meter3(0.0), Meter3(5.0))
                    .value(self.runoff)
                    .normalize()
                    // Runoff doesn't have a fixed range so we have to clamp
                    // this to make sure we don't overflow the color value
                    .clamp()
                    .convert::<f64>()
                    .inner() as f32;
                // 0 -> white
                // 1 -> blue
                Color3::new(1.0 - normal_runoff, 1.0 - normal_runoff, 1.0)
            }
        }
        .unwrap()
    }
}

impl HasHexPosition for Tile {
    fn position(&self) -> HexPoint {
        self.position
    }
}

/// A definition of what data is used to compute a tile's color.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum TileLens {
    Biome,
    Elevation,
    Humidity,
    Runoff,
}

// Types that we can't natively return. These are assigned TS types, but
// these types aren't actually verified by the compiler. Be careful
// here!
#[wasm_bindgen]
extern "C" {

    /// Type hack needed until https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen(typescript_type = "Tile[]")]
    pub type TileArray;
}
