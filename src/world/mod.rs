mod generate;

use crate::util::Color3;
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HexPoint {
    x: isize,
    y: isize,
    z: isize,
}

impl HexPoint {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        if x + y + z != 0 {
            panic!(
                "Sum of coordinates should be zero, but got ({}, {}, {})",
                x, y, z
            );
        }
        Self { x, y, z }
    }

    pub fn get_pixel_pos(&self, scale: f64) -> (f64, f64) {
        // final double x = TILE_WIDTH * tilePos.x() * 0.75;
        // final double y = -TILE_DEPTH * (tilePos.x() / 2.0 + tilePos.y());
        // return new Point2(x, y);

        let pixel_x: f64 = self.x as f64 * 0.75;
        let pixel_y: f64 =
            (self.x as f64 / 2.0 + self.y as f64) * -(3.0_f64.sqrt() / 2.0);
        (pixel_x * scale, pixel_y * scale)
    }
}

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
    fn biome_type(self) -> BiomeType {
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

    fn get_color(self) -> Color3 {
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
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tile {
    pub position: HexPoint,
    pub elevation: f64,
    pub humidity: f64,
    pub biome: Biome,
}

impl Tile {
    pub fn get_color(&self) -> Color3 {
        self.biome.get_color()
    }
}

pub struct World {
    tiles: BTreeMap<HexPoint, Tile>,
}
