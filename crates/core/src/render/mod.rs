pub mod config;

use crate::{
    render::config::RenderConfig, Biome, BiomeType, GeoFeature, Meter3,
    NumRange, Tile, World,
};
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Neg,
    Sub, SubAssign, Sum,
};
use serde::{Deserialize, Serialize};
use std::ops;
use strum::EnumString;
use validator::Validate;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A world renderer is used to convert worlds into various visual output
/// formats. A renderer is created using a particular [RenderConfig], and from
/// there can be used to render any number of worlds any number of times.
///
/// Aside from providing complete rendering, a renderer also provides utility
/// functions that make it easy to generate your own renderings based on a
/// world.
///
/// Config options cannot be changed after creating a renderer, but renderers
/// are very cheap to create so if you need to change the config, just create
/// a new renderer.
///
/// ## Supported Formats
/// - STL (3D, no colors or textures)
/// - SVG (2D with colors and textures)
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldRenderer {
    /// Config that determines how rendering is done
    ///
    /// **This is different from the world config.** The world config controls
    /// how the world is generated the render config just controls how it's
    /// visually presented _after_ generation.
    render_config: RenderConfig,
}

// Non-Wasm API
impl WorldRenderer {
    /// Initialize a new renderer with the given options. Returns an error if
    /// the render config is invalid.
    pub fn new(render_config: RenderConfig) -> anyhow::Result<Self> {
        render_config.validate()?;
        Ok(Self { render_config })
    }

    /// Get a reference to the config that this renderer uses
    pub fn render_config(&self) -> &RenderConfig {
        &self.render_config
    }
}

// Wasm-friendly API
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WorldRenderer {
    /// Get the height that a tile's geometry should have. This will convert
    /// the tile's elevation to a zero-based scale, then multiplicatively scale
    /// it based on the pre-configured Y scale of the world. See
    /// [RenderConfig::vertical_scale] for more info on what exactly the
    /// vertical scale means.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn tile_height(&self, tile: &Tile) -> f64 {
        // Map elevation to a zero-based scale
        let zeroed_elevation = World::ELEVATION_RANGE
            .map_to(&World::ELEVATION_RANGE.zeroed(), tile.elevation());
        // Multiply by render scale
        zeroed_elevation.0 * self.render_config.vertical_scale
    }

    /// Compute the color of a tile based on current render settings. The tile
    /// lens in the render config controls what data the color is derived from.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn tile_color(&self, tile: &Tile) -> Color3 {
        match self.render_config.tile_lens {
            // See TileLens definition for a description of each lens type
            TileLens::Surface => {
                if tile.features().contains(&GeoFeature::Lake) {
                    Color3::new_int(72, 192, 240)
                } else {
                    self.biome_color(tile.biome())
                }
            }
            TileLens::Biome => self.biome_color(tile.biome()),
            TileLens::Elevation => {
                let normal_elev =
                    World::ELEVATION_RANGE.normalize(tile.elevation()).0 as f32;
                // 0 -> white
                // 1 -> red
                Color3::new(1.0, 1.0 - normal_elev, 1.0 - normal_elev)
            }
            TileLens::Humidity => {
                let humidity = tile.humidity() as f32;
                // 0 -> white
                // 1 -> green
                Color3::new(1.0 - humidity, 1.0, 1.0 - humidity)
            }
            TileLens::Runoff => {
                // This coloring is based on two aspects: runoff (how much water
                // collected on the tile) AND runoff egress (how much water
                // flowed over the tile without staying there). Runoff controls
                // blue, runoff egress controls green.
                if tile.biome().biome_type() == BiomeType::Water {
                    Color3::new(0.5, 0.5, 0.5)
                } else {
                    // Neither value we use here has a hard cap, so we use
                    // arbitrary max values based on what's common/reasonable,
                    // and anything over that will just be clamped down
                    let normal_runoff = NumRange::new(Meter3(0.0), Meter3(5.0))
                        .value(tile.runoff())
                        .normalize()
                        .clamp()
                        .convert::<f64>()
                        .inner() as f32;
                    let normal_runoff_egress =
                        NumRange::new(Meter3(0.0), Meter3(1000.0))
                            .value(tile.runoff_egress())
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
    }

    /// Map a biome to its preset color.
    pub fn biome_color(&self, biome: Biome) -> Color3 {
        match biome {
            Biome::Ocean => Color3::new_int(20, 77, 163),
            Biome::Coast => Color3::new_int(32, 166, 178),

            Biome::Snow => Color3::new_int(191, 191, 191),
            Biome::Desert => Color3::new_int(214, 204, 107),
            Biome::Alpine => Color3::new_int(99, 122, 99),
            Biome::Jungle => Color3::new_int(43, 179, 31),
            Biome::Forest => Color3::new_int(23, 122, 0),
            Biome::Plains => Color3::new_int(173, 201, 115),
        }
    }

    /// Render this world as a 2D SVG, from a top-down perspective. Returns the
    /// SVG in a string.
    #[cfg(feature = "svg")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn render_as_svg(&self, world: &World) -> String {
        use crate::util;
        let svg = util::svg::world_to_svg(world, self);
        svg.to_string()
    }

    /// Render this world into an STL model. Return value is the STL binary
    /// data. Returns an error if serialization fails, which indicates a bug
    /// in terra or stl_io.
    #[cfg(feature = "stl")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn render_as_stl(&self, world: &World) -> Vec<u8> {
        use crate::util;
        let mesh = util::stl::world_to_stl(world, self);
        let mut buffer = Vec::<u8>::new();
        // Panic here indicates a bug in our STL mesh format
        stl_io::write_stl(&mut buffer, mesh.iter())
            .expect("error serializing STL");
        buffer
    }
}

/// A point in 2D rendered space. This isn't used at all during world
/// generation/processing, but is useful during rendering. You can use
/// [HexPoint::to_point2] to convert a tile's world position into a renderable
/// 2D position. These positions aren't really useful outside of rendering, so
/// stick to [HexPoint] for stuff like distances, pathfinding, etc.
///
/// ## 2D Coordinates
///
/// Unlike hex coordinates, which have 3 components, 2D coordinates obviously
/// only have 2. This coordinate system uses the center of the screen as the
/// origin, so the tile with the hex position of `(0, 0, 0)` will be centered on
/// `(0, 0)` in 2D. Left is negative x, right is positive x. Down is positive y,
/// up is negative y.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Display,
    PartialEq,
    PartialOrd,
    From,
    Into,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Sum,
)]
#[display(fmt = "({}, {})", x, y)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

/// A vector in 2D space. Like [Point2], this isn't used during world generation
/// at all, but is useful during rendering. This can represent offsets in 2D.
///
/// See [Point2] for a description of the 2D coordinate space.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Display,
    PartialEq,
    PartialOrd,
    From,
    Into,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Sum,
)]
#[display(fmt = "({}, {})", x, y)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl ops::Add<Vector2> for Point2 {
    type Output = Point2;

    fn add(self, rhs: Vector2) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// An RGB color. Values are stored as floats between 0 and 1 (inclusive).
/// This uses f32 because the extra precision from f64 is pointless.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color3 {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color3 {
    /// The valid range of values for each component in RGB
    const COMPONENT_RANGE: NumRange<f32> = NumRange::new(0.0, 1.0);

    /// Create a new RGB color with components in the range [0.0, 1.0]. Panic
    /// if any of the components are out of range
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        fn check_component(component_name: &str, value: f32) -> f32 {
            if Color3::COMPONENT_RANGE.contains(value) {
                value
            } else {
                panic!(
                    "Color component {} must be in {}, but was {}",
                    component_name,
                    Color3::COMPONENT_RANGE,
                    value
                )
            }
        }

        Self {
            red: check_component("red", red),
            green: check_component("green", green),
            blue: check_component("blue", blue),
        }
    }

    /// Create a new RGB color from integer components in the [0,255] range.
    pub const fn new_int(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
        }
    }

    /// Convert this number to a set of 3 bytes: `(red, green, blue)`
    pub fn to_ints(self) -> (u8, u8, u8) {
        (
            (self.red * 255.0) as u8,
            (self.green * 255.0) as u8,
            (self.blue * 255.0) as u8,
        )
    }

    /// Convert this color to an HTML color code: `#rrggbb`
    pub fn to_html(self) -> String {
        let (r, g, b) = self.to_ints();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

// Scale a color by a constant
impl ops::Mul<f32> for Color3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        let red = Self::COMPONENT_RANGE.clamp(self.red * rhs);
        let green = Self::COMPONENT_RANGE.clamp(self.green * rhs);
        let blue = Self::COMPONENT_RANGE.clamp(self.blue * rhs);
        // It's safe to bypass the constructor here because we just clamped
        // all 3 components to the valid range
        Self { red, green, blue }
    }
}

/// A definition of what data is used to compute a tile's color.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
