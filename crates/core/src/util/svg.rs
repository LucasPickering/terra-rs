use crate::{
    render::{Color3, WorldRenderer},
    GeoFeature, HasHexPosition, HexAxialDirection, Tile, World,
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
/// color
pub fn world_to_svg(world: &World, renderer: &WorldRenderer) -> Document {
    // Set the view box based on the world size. Each of these values is the
    // distance from the center of the viewbox to the outer edge. So the
    // width/height will be double that value
    let radius = world.config().radius as f64;
    // Distance from center of origin tile to center of right-most tile,
    // **plus** the center of that right-most tile to its right-most edge
    let view_box_max_x =
        (radius * Tile::CENTER_DISTANCE_X + Tile::VERTEX_RADIUS).ceil();
    // Distance from the center of origin tile to center of bottom-most tile,
    // **plus** the center of that bottom-most tile to its bottom edge
    let view_box_max_y =
        (radius * Tile::CENTER_DISTANCE_Y + Tile::SIDE_RADIUS).ceil();

    let mut document = Document::new()
        .set(
            "viewBox",
            (
                // Top-left corner
                -view_box_max_x,
                -view_box_max_y,
                // Width and height
                view_box_max_x * 2.0,
                view_box_max_y * 2.0,
            ),
        )
        .set("shape-rendering", "crispEdges")
        .add(Comment::new(format!("\n{:#?}\n", world.config())));

    for tile in world.tiles().values() {
        let polygon = draw_tile(renderer, tile);
        document = document.add(polygon);
    }

    document
}

/// Generate an SVG polygon for a single tile
fn draw_tile(world_renderer: &WorldRenderer, tile: &Tile) -> Group {
    let pos = tile.position();
    let pos2d = pos.to_point2();

    // Start with the main tile hexagon
    let mut group = Group::new()
        // Translate the tile to its correct position
        .set("transform", format!("translate({} {})", pos2d.x, pos2d.y))
        .add(Comment::new(pos.to_string())) // Readability!
        .add(
            Polygon::new()
                // Generate vertices for the tile. This attribute ends up being
                // the same for every tile, but we can't really pull this code
                // out because the SVG lib forces us to clone the vec every
                // time anyway. So it's just easier to leave it like this
                .set(
                    "points",
                    HexAxialDirection::CLOCKWISE
                        .iter()
                        .map(|dir| {
                            let v = dir.to_vector2();
                            (v.x, v.y)
                        })
                        .collect::<Vec<_>>(),
                )
                // Set color
                .set("fill", world_renderer.tile_color(tile).to_html()),
        );

    // Add overlays for each geo feature
    if world_renderer.render_config().show_features {
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
