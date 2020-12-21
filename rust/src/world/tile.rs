use crate::{
    util::{Color3, NumRange},
    world::{
        hex::{HasHexPosition, HexPoint, HexPointMap},
        Biome, World,
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct Tile {
    position: HexPoint,
    elevation: f64,
    humidity: f64,
    runoff_acc: f64,
    biome: Biome,
}

#[wasm_bindgen]
impl Tile {
    #[wasm_bindgen(getter)]
    pub fn pos(&self) -> HexPoint {
        self.position
    }

    #[wasm_bindgen(getter)]
    pub fn elevation(&self) -> f64 {
        self.elevation
    }

    /// Tile elevation, but mapped to a zero-based range so the value is
    /// guaranteed to be non-negative. This makes it safe to use for vertical
    /// scaling during rendering.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> f64 {
        World::ELEVATION_RANGE
            .map(&World::ELEVATION_RANGE.zeroed(), self.elevation)
    }

    #[wasm_bindgen(getter)]
    pub fn humidity(&self) -> f64 {
        self.humidity
    }

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
                    World::ELEVATION_RANGE.normalize(self.elevation()) as f32;
                // 0 -> white
                // 1 -> red
                Color3::new(1.0, 1.0 - normal_elev, 1.0 - normal_elev)
            }
            TileLens::Humidity => {
                let normal_humidity =
                    World::HUMIDITY_RANGE.normalize(self.humidity()) as f32;
                // 0 -> white
                // 1 -> green
                Color3::new(1.0 - normal_humidity, 1.0, 1.0 - normal_humidity)
            }
            TileLens::Runoff => {
                let normal_runoff = NumRange::new(0.0, 10.0)
                    .value(self.runoff_acc)
                    .normalize()
                    .clamp()
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

/// A partially built [Tile]. This should only be used while the world is being
/// generated. After generation is complete, only [Tile] should be used. All the
/// fields on this type, other than `position`, have a getter and a setter.
/// Since the fields may not be defined, the getters all panic if the field
/// has not be set. This makes it easy to catch bugs where we're trying to use
/// world values that haven't been generated yet.
#[derive(Copy, Clone, Debug)]
pub struct TileBuilder {
    position: HexPoint,
    elevation: Option<f64>,
    humidity: Option<f64>,
    biome: Option<Biome>,
    runoff_acc: f64,
}

impl TileBuilder {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            elevation: None,
            humidity: None,
            biome: None,
            runoff_acc: 0.0,
        }
    }

    pub fn build(self) -> Tile {
        Tile {
            position: self.position,
            elevation: self.elevation.unwrap(),
            humidity: self.humidity.unwrap(),
            biome: self.biome.unwrap(),
            runoff_acc: self.runoff_acc,
        }
    }

    /// Get this tile's elevation. Panics if elevation has not been set yet.
    pub fn elevation(&self) -> Option<f64> {
        self.elevation
    }

    /// Set the elevation for this tile.
    pub fn set_elevation(&mut self, elevation: f64) {
        self.elevation = Some(elevation);
    }

    /// Get this tile's humidity. Panics if humidity has not been set yet.
    pub fn humidity(&self) -> Option<f64> {
        self.humidity
    }

    /// Set the humidity for this tile.
    pub fn set_humidity(&mut self, humidity: f64) {
        self.humidity = Some(humidity);
    }

    /// Get this tile's biome. Panics if biome has not been set yet.
    pub fn biome(&self) -> Option<Biome> {
        self.biome
    }

    /// Set the biome for this tile.
    pub fn set_biome(&mut self, biome: Biome) {
        self.biome = Some(biome);
    }
}

impl HasHexPosition for TileBuilder {
    fn position(&self) -> HexPoint {
        self.position
    }
}

pub type TileMap = HexPointMap<Tile>;

/// A definition of what data is used to compute a tile's color.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum TileLens {
    Biome,
    Elevation,
    Humidity,
    Runoff,
}
