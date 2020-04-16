mod generate;

use crate::{
    util::{Color3, FloatRange},
    world::generate::WorldBuilder,
};
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

    /// Convert this hexagonal coordinate into a 2d pixel coordinate. Useful
    /// for figuring out where to position a tile on the screen.
    pub fn get_pixel_pos(&self, scale: f64) -> (f64, f64) {
        let pixel_x: f64 = self.x as f64 * 0.75;
        let pixel_y: f64 =
            (self.x as f64 / 2.0 + self.y as f64) * -(3.0_f64.sqrt() / 2.0);
        (pixel_x * scale, pixel_y * scale)
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
    pub const ELEVATION_RANGE: FloatRange = FloatRange::new(-50.0, 50.0);
    pub const HUMDITY_RANGE: FloatRange = FloatRange::NORMAL_RANGE;

    pub fn color(&self) -> Color3 {
        self.biome.color()
    }
}

impl HasHexPosition for Tile {
    fn position(&self) -> HexPoint {
        self.position
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WorldConfig {
    pub seed: u32,
    pub tile_radius: usize,
}

#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    tiles: HexPointMap<Tile>,
}

impl World {
    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.values()
    }

    pub fn generate(config: WorldConfig) -> Self {
        let tiles = WorldBuilder::new(config).generate_world();
        Self { config, tiles }
    }
}
