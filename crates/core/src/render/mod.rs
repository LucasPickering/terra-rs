pub mod config;
#[cfg(feature = "stl")]
pub mod stl;
#[cfg(feature = "svg")]
pub mod svg;
pub mod unit;

use crate::{
    render::{
        config::RenderConfig,
        unit::{Color3, Point2},
    },
    world::hex::HexThing,
    Biome, BiomeType, GeoFeature, HasHexPosition, Meter, Meter3, NumRange,
    Tile, World,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::f64;
use strum::EnumString;
use validator::Validate;
#[cfg(feature = "js")]
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
#[cfg_attr(feature = "js", wasm_bindgen)]
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
    // Rendering constants below
    /// Distance between the center of a tile and one of its 6 vertices, in
    /// **screen space**. This is also the length of one side of the tile.
    ///
    /// ## Rendering Constant Caveat
    /// This value is **not** consistent with the abstract units of [Meter]/
    /// [Meter2]/[Meter3]. There is some artistic license employed during
    /// rendering. See [crate::hex] for a description of what screen space is.
    pub const TILE_VERTEX_RADIUS: f64 = 1.0;
    /// Distance between the center of a tile and the midpoint of one of its
    /// sides, in **2D space**. See [Self::TILE_VERTEX_RADIUS] for the rendering
    /// constant caveat.
    pub const TILE_SIDE_RADIUS: f64 = Self::TILE_VERTEX_RADIUS * 0.8660254; // sqrt(3)/2
    /// Distance between any two opposite **vertices** of a tile, in **2D
    /// space**. See [Self::TILE_VERTEX_RADIUS] for the rendering constant
    /// caveat.
    pub const TILE_WIDTH: f64 = Self::TILE_VERTEX_RADIUS * 2.0;
    /// Distance between any two opposite **sides** of a tile, in **2D space**.
    /// See [Self::TILE_VERTEX_RADIUS] for the rendering constant caveat.
    pub const TILE_HEIGHT: f64 = Self::TILE_SIDE_RADIUS * 2.0;
    /// Distance **in the X axis only** between the center of two tiles that are
    /// aligned in the Y and one unit apart in the X (i.e. left-to-right).
    /// See [Self::TILE_VERTEX_RADIUS] for the rendering constant caveat.
    pub const TILE_CENTER_DISTANCE_X: f64 = Self::TILE_VERTEX_RADIUS * 1.5;
    /// Distance between the center of two tiles that are aligned in the X
    /// and one unit apart in the Y (i.e. up-and-down). See
    /// [Self::TILE_VERTEX_RADIUS] for the rendering constant caveat.
    pub const TILE_CENTER_DISTANCE_Y: f64 = Self::TILE_HEIGHT;

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

    /// Convert a point from from hex space to 2D screen space. Useful for
    /// rendering tiles or other world objects into a visual format.
    pub fn hex_to_screen_space<T: Into<f64>>(
        &self,
        point: impl HexThing<Component = T>,
    ) -> Point2 {
        // This is a simplification of some linear algebra. We need to apply
        // three transformations, in sequence:
        // 1. Project onto the plane x+y+z=0
        // 2. Rotate 45° CCW around z
        // 3. Rotate 45° CCW around x
        // This should leave us with a bunch of points on the plane z=0
        // If you create a 3x3 matrix each transformation, then multiply them
        // together, you get this matrix:
        // +-                             -+
        // |     √2/2     -√2/2          0 |
        // | (1+√2)/6  (1+√2)/6  (-1-√2)/3 |
        // | (1-√2)/6  (1-√2)/6  (-1+√2)/3 |
        // +-                             -+
        // The math below is just multiplying the vector (x,y,z) by that matrix,
        // then throwing away the third component to get a 2D (x,y). TBH I'm not
        // sure why the z component doesn't just spit out 0 anyway, since this
        // is supposed to be the plane z=0. But the math works so I'm not gonna
        // question it too much.

        let x: f64 = point.x().into();
        let y: f64 = point.y().into();
        let z: f64 = point.z().into();
        Point2 {
            x: 2.0f64.sqrt() / 2.0 * x - 2.0f64.sqrt() / 2.0 * y,
            y: (1.0 + 2.0f64.sqrt()) / 6.0 * x
                + (1.0 + 2.0f64.sqrt()) / 6.0 * y
                + (-1.0 - 2.0f64.sqrt()) / 3.0 * z,
        }
    }
}

// Wasm-friendly API
#[cfg_attr(feature = "js", wasm_bindgen)]
impl WorldRenderer {
    /// Get the position of a tile, in screen space. See the module-level doc
    /// at [crate::hex] for a description of screen coordinate space.
    pub fn tile_position(&self, tile: &Tile) -> Point2 {
        self.hex_to_screen_space(tile.position())
    }

    /// Get the distance between the center of a tile and the midpoint of one
    /// of its sides. Useful for scaling tiles in certain render contexts.
    pub fn tile_side_radius(&self) -> f64 {
        Self::TILE_SIDE_RADIUS
    }

    /// Get the height that a tile's geometry should have. This will convert
    /// the tile's elevation to a zero-based scale, then multiplicatively scale
    /// it based on the pre-configured Y scale of the world. See
    /// [RenderConfig::vertical_scale] for more info on what exactly the
    /// vertical scale means.
    pub fn tile_height(&self, tile: &Tile) -> f64 {
        self.elevation_to_height(tile.elevation())
    }

    /// Get the height of sea level, in absolute coordinates
    pub fn sea_level_height(&self) -> f64 {
        self.elevation_to_height(World::SEA_LEVEL)
    }

    /// Convert a relative elevation value to an absolute height, to be used
    /// in 3D rendering coordinates.
    pub fn elevation_to_height(&self, elevation: Meter) -> f64 {
        // Map elevation to a zero-based scale
        let zeroed_elevation = World::ELEVATION_RANGE
            .map_to(&World::ELEVATION_RANGE.zeroed(), elevation);
        // Multiply by render scale
        zeroed_elevation.0 * self.render_config.vertical_scale
    }

    /// Compute the color of a tile based on current render settings. The tile
    /// lens in the render config controls what data the color is derived from.
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
                    // TODO make max runoff configurable here
                    let normal_runoff =
                        self.normalize_runoff(tile.runoff()) as f32;
                    let normal_runoff_egress =
                        self.normalize_runoff_flow(tile.runoff_egress()) as f32;

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

    /// Normalize a runoff value into the range `[0, 1]`. Since runoff values
    /// have no hard upper bound, this function relies on a soft bound from
    /// the render config to determine what value maps to `1`. Any runoff
    /// flow value at or above [RenderConfig::max_runoff] will map to
    /// `1`. Everything less than that will map proportionally between `0`
    /// and `1`.
    pub fn normalize_runoff(&self, runoff: Meter3) -> f64 {
        NumRange::new(Meter3(0.0), self.render_config.max_runoff)
            .value(runoff)
            .normalize()
            .clamp()
            .convert::<f64>()
            .inner()
    }

    /// Normalize a runoff flow value (i.e. either runoff ingress or runoff
    /// egress) into the range `[0, 1]`. Since runoff values have no hard upper
    /// bound, this function relies on a soft bound from the render config to
    /// determine what value maps to `1`. Any runoff flow value at or above
    /// [RenderConfig::max_runoff_flow] will map to `1`. Everything less than
    /// that will map proportionally between `0` and `1`.
    pub fn normalize_runoff_flow(&self, runoff_flow: Meter3) -> f64 {
        NumRange::new(Meter3(0.0), self.render_config.max_runoff_flow)
            .value(runoff_flow)
            .normalize()
            .clamp()
            .convert::<f64>()
            .inner()
    }

    /// Render this world as a 2D SVG, from a top-down perspective. Returns the
    /// SVG in a string.
    #[cfg(feature = "svg")]
    pub fn render_as_svg(&self, world: &World) -> String {
        let svg = svg::world_to_svg(world, self);
        svg.to_string()
    }

    /// Render this world into an STL model. Return value is the STL binary
    /// data. Returns an error if serialization fails, which indicates a bug
    /// in terra or stl_io.
    #[cfg(feature = "stl")]
    pub fn render_as_stl(&self, world: &World) -> Vec<u8> {
        let mesh = stl::world_to_stl(world, self);
        let mut buffer = Vec::<u8>::new();
        // Panic here indicates a bug in our STL mesh format
        stl_io::write_stl(&mut buffer, mesh.iter())
            .expect("error serializing STL");
        buffer
    }
}

/// A definition of what data is used to compute a tile's color.
#[cfg_attr(feature = "js", wasm_bindgen)]
#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    Eq,
    PartialEq,
    EnumString,
    Serialize,
    Deserialize,
)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{world::hex::TileVertexPoint, TilePoint};
    use assert_approx_eq::assert_approx_eq;

    impl Point2 {
        /// Allows us to use assert_approx_eq
        #[cfg(test)]
        pub fn abs(self) -> f64 {
            self.x.abs() + self.y.abs()
        }
    }

    #[test]
    fn test_hex_to_screen_space() {
        let renderer = WorldRenderer::new(RenderConfig::default()).unwrap();

        // Tile points
        assert_approx_eq!(
            renderer.hex_to_screen_space(TilePoint::new_xy(0, 0)),
            Point2::new(0.0, 0.0)
        );
        assert_approx_eq!(
            renderer.hex_to_screen_space(TilePoint::new_xy(1, -2)),
            Point2::new(2.121320, -1.207107)
        );
        assert_approx_eq!(
            renderer.hex_to_screen_space(TilePoint::new_xy(1, 1)),
            Point2::new(0.0, 2.414214)
        );
        assert_approx_eq!(
            renderer.hex_to_screen_space(TilePoint::new_xy(-10, -3)),
            Point2::new(-4.949747, -15.692388)
        );

        // Tile vertex points
        assert_approx_eq!(
            renderer
                .hex_to_screen_space(TileVertexPoint::new(-3, 0, 1).unwrap()),
            Point2::new(-2.121320, -2.011844)
        );
        assert_approx_eq!(
            renderer
                .hex_to_screen_space(TileVertexPoint::new(4, -4, -2).unwrap()),
            Point2::new(5.656854, 1.609476)
        );
    }
}
