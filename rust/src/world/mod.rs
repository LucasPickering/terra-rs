mod generate;

use crate::{
    config::WorldConfig,
    util::{Color3, NumRange},
    world::generate::WorldBuilder,
};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HexPoint {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl HexPoint {
    /// Construct a new hex point with the given x and y. Since x+y+z=0 for all
    /// points, we can derive z from x & y.
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y, z: -(x + y) }
    }

    pub fn x(&self) -> isize {
        self.x
    }

    pub fn y(&self) -> isize {
        self.y
    }

    pub fn z(&self) -> isize {
        self.z
    }

    /// Convert this hexagonal coordinate into a 2d pixel coordinate. Useful
    /// for figuring out where to position a tile on the screen.
    pub fn pixel_pos(&self, scale: f32) -> (f32, f32) {
        let pixel_x: f32 = self.x as f32 * 0.75;
        let pixel_y: f32 =
            (self.x as f32 / 2.0 + self.y as f32) * -(3.0_f32.sqrt() / 2.0);
        (pixel_x * scale, pixel_y * scale)
    }
}

impl Display for HexPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

pub trait HasHexPosition: Sized {
    fn position(&self) -> HexPoint;

    /// Convert this value into a tuple with the position. Useful when mapping
    /// an iterator then collecting into a [HexPointMap].
    fn into_tuple(self) -> (HexPoint, Self) {
        (self.position(), self)
    }
}

pub type HexPointMap<T> = BTreeMap<HexPoint, T>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BiomeType {
    Water,
    Land,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Biome {
    Ocean,
    Coast,
    Lake,

    Snow,
    Desert,
    Alpine,
    Jungle,
    Forest,
    Plains,
    Beach,
}

impl Biome {
    fn _biome_type(self) -> BiomeType {
        match self {
            Self::Ocean | Self::Coast | Self::Lake => BiomeType::Water,
            Self::Snow
            | Self::Desert
            | Self::Alpine
            | Self::Jungle
            | Self::Forest
            | Self::Plains
            | Self::Beach => BiomeType::Land,
        }
    }

    fn color(self) -> Color3 {
        match self {
            Self::Ocean => Color3::new(0.08, 0.30, 0.64),
            Self::Coast => Color3::new(0.22, 0.55, 0.78),
            Self::Lake => Color3::new(0.04, 0.55, 0.75),

            Self::Snow => Color3::new(0.75, 0.75, 0.75),
            Self::Desert => Color3::new(0.84, 0.80, 0.42),
            Self::Alpine => Color3::new(0.39, 0.48, 0.37),
            Self::Jungle => Color3::new(0.17, 0.70, 0.12),
            Self::Forest => Color3::new(0.09, 0.48, 0.0),
            Self::Plains => Color3::new(0.68, 0.79, 0.45),
            Self::Beach => Color3::new(0.95, 0.94, 0.35),
        }
        .unwrap()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tile {
    position: HexPoint,
    elevation: f64,
    humidity: f64,
    biome: Biome,
}

impl Tile {
    pub const ELEVATION_RANGE: NumRange<f64> = NumRange::new(-50.0, 50.0);
    pub const HUMDITY_RANGE: NumRange<f64> = NumRange::new(0.0, 1.0);

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
                    Self::HUMDITY_RANGE.normalize(self.humidity()) as f32;
                Color3::new(normal_humidity, normal_humidity, 1.0)
            }
        }
        .unwrap()
    }
}

/// A definition of what data is used to compute a tile's color.
#[derive(Copy, Clone, Debug)]
pub enum TileLens {
    Composite,
    Elevation,
    Humidity,
    Biome,
}

impl HasHexPosition for Tile {
    fn position(&self) -> HexPoint {
        self.position
    }
}

#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    tiles: HexPointMap<Tile>,
}

impl World {
    pub fn tiles(&self) -> &HexPointMap<Tile> {
        &self.tiles
    }

    pub fn generate(config: WorldConfig) -> Self {
        let tiles = WorldBuilder::new(config).generate_world();
        Self { config, tiles }
    }
}
