use crate::{
    Color3, GeoFeature, HasHexPosition, HexAxialDirection, Tile, TileLens,
    World,
};
use svg::{
    node::{
        element::{Group, Line, Polygon},
        Comment,
    },
    Document,
};

const RIVER_COLOR: Color3 = Color3::new_int(72, 192, 240);

/// Render a world as an SVG. This will be a 2D top-down rendering, in full
/// color.
///
/// ## Params
/// - `world` - The world to render
/// - `lens` - The [TileLens] to use when determining each tile's color
/// - `show_features` - Should geographic features (lakes, rivers, etc.) be
///   rendered? See [crate::GeoFeature] for a full list
pub fn world_to_svg(
    world: &World,
    lens: TileLens,
    show_features: bool,
) -> Document {
    // Grow the view box based on the world size. The world height will always
    // be the larger size, so scale it based on that. The +1 provides a bit of
    // buffer space
    let view_box_size =
        ((world.config().radius as f64 + 1.0) * Tile::HEIGHT).ceil();
    let mut document = Document::new()
        .set(
            "viewBox",
            (
                -view_box_size,
                -view_box_size,
                view_box_size * 2.0,
                view_box_size * 2.0,
            ),
        )
        .set("shape-rendering", "crispEdges")
        .add(Comment::new(format!("\n{:#?}\n", world.config())));

    for tile in world.tiles().values() {
        let polygon = draw_tile(tile, lens, show_features);
        document = document.add(polygon);
    }

    document
}

/// Generate an SVG polygon for a single tile
fn draw_tile(tile: &Tile, lens: TileLens, show_features: bool) -> Group {
    let pos = tile.position();
    let pos2d = pos.to_point2();

    // Start with the main tile hexagon
    let mut group = Group::new()
        .set("transform", format!("translate({} {})", pos2d.x, pos2d.y))
        .add(Comment::new(pos.to_string())) // Readability!
        .add(
            Polygon::new()
                .set(
                    "points",
                    HexAxialDirection::ALL
                        .iter()
                        .map(|dir| {
                            let v = dir.to_vector2();
                            (v.x, v.y)
                        })
                        .collect::<Vec<_>>(),
                )
                .set("fill", lens.tile_color(tile).to_html()),
        );

    // Add overlays for each geo feature
    if show_features {
        for feature in tile.features() {
            match feature {
                GeoFeature::Lake => {} // This is covered by TileLens::Surface
                GeoFeature::RiverEntrance { direction, .. }
                | GeoFeature::RiverExit { direction, .. } => {
                    let side_offset = direction.to_vector2();
                    group = group.add(
                        Line::new()
                            .set("x1", 0)
                            .set("y1", 0)
                            .set("x2", side_offset.x)
                            .set("y2", side_offset.y)
                            .set("stroke", RIVER_COLOR.to_html())
                            .set("stroke-width", 0.4),
                    );
                }
            }
        }
    }

    group
}
