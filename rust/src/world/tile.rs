use crate::{
    util::{Color3, NumRange},
    world::{
        hex::{HasHexPosition, HexPoint, HexPointMap},
        Biome,
    },
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tile {
    position: HexPoint,
    elevation: f64,
    humidity: f64,
    biome: Biome,
}

impl Tile {
    pub const ELEVATION_RANGE: NumRange<f64> = NumRange::new(-50.0, 50.0);
    pub const HUMIDITY_RANGE: NumRange<f64> = NumRange::new(0.0, 1.0);

    pub fn elevation(&self) -> f64 {
        self.elevation
    }

    pub fn humidity(&self) -> f64 {
        self.humidity
    }

    pub fn biome(&self) -> Biome {
        self.biome
    }

    /// Compute the color of a tile based on the lens being viewed. The lens
    /// controls what data the color is derived from.
    pub fn color(&self, lens: TileLens) -> Color3 {
        match lens {
            TileLens::Composite => {
                let normal_elev =
                    Self::ELEVATION_RANGE.normalize(self.elevation()) as f32;
                Ok(self.biome().color() * normal_elev)
            }
            TileLens::Biome => Ok(self.biome.color()),
            TileLens::Elevation => {
                let normal_elev =
                    Self::ELEVATION_RANGE.normalize(self.elevation()) as f32;
                Color3::new(1.0, normal_elev, normal_elev)
            }
            TileLens::Humidity => {
                let normal_humidity =
                    Self::HUMIDITY_RANGE.normalize(self.humidity()) as f32;
                Color3::new(normal_humidity, normal_humidity, 1.0)
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
}

impl TileBuilder {
    pub fn new(position: HexPoint) -> Self {
        Self {
            position,
            elevation: None,
            humidity: None,
            biome: None,
        }
    }

    pub fn build(self) -> Tile {
        Tile {
            position: self.position(),
            elevation: self.elevation(),
            humidity: self.humidity(),
            biome: self.biome(),
        }
    }

    /// Get this tile's elevation. Panics if elevation has not been set yet.
    pub fn elevation(&self) -> f64 {
        self.elevation.unwrap()
    }

    /// Set the elevation for this tile.
    pub fn set_elevation(&mut self, elevation: f64) {
        self.elevation = Some(elevation);
    }

    /// Get this tile's humidity. Panics if humidity has not been set yet.
    pub fn humidity(&self) -> f64 {
        self.humidity.unwrap()
    }

    /// Set the humidity for this tile.
    pub fn set_humidity(&mut self, humidity: f64) {
        self.humidity = Some(humidity);
    }

    /// Get this tile's biome. Panics if biome has not been set yet.
    pub fn biome(&self) -> Biome {
        self.biome.unwrap()
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
#[derive(Copy, Clone, Debug)]
pub enum TileLens {
    Composite,
    Elevation,
    Humidity,
    Biome,
}
