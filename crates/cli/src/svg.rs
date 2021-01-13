use svg::{
    node::{element::Polygon, Comment},
    Document,
};
use terra::{HasHexPosition, Tile, TileLens, World};

/// Distance between the center of a tile and one of its 6 vertices
const TILE_VERTEX_RADIUS: f32 = 1.0;
/// Distance between the center of a tile and the midpoint of one of its sides
const TILE_SIDE_RADIUS: f32 = TILE_VERTEX_RADIUS * 0.8660254; // sqrt(3)/2

/// Distance between the bottom side and top side of a tile.
const TILE_HEIGHT: f32 = TILE_SIDE_RADIUS * 2.0;

/// Coordinates of each vertex of the tile, relative to its center. Starts with
/// the left vertex and goes clockwise from there.
const TILE_VERTICES: &[(f32, f32)] = &[
    (-TILE_VERTEX_RADIUS, 0.0),                     // Left
    (-TILE_VERTEX_RADIUS / 2.0, -TILE_SIDE_RADIUS), // Top-left
    (TILE_VERTEX_RADIUS / 2.0, -TILE_SIDE_RADIUS),  // Top-right
    (TILE_VERTEX_RADIUS, 0.0),                      // Right
    (TILE_VERTEX_RADIUS / 2.0, TILE_SIDE_RADIUS),   // Bottom-right
    (-TILE_VERTEX_RADIUS / 2.0, TILE_SIDE_RADIUS),  // Bottom-left
];

/// Generate an SVG document for a world
pub fn draw_world(world: &World) -> Document {
    // Grow the view box based on the world size. The world height will always
    // be the larger size, so scale it based on that. The +1 provides a bit of
    // buffer space
    let view_box_size =
        ((world.config().radius as f32 + 1.0) * TILE_HEIGHT).ceil();
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
        let polygon = draw_tile(tile);
        document = document.add(polygon);
    }

    document
}

/// Generate an SVG polygon for a single tile
fn draw_tile(tile: &Tile) -> Polygon {
    let pos = tile.position();
    Polygon::new()
        .set("points", TILE_VERTICES.iter().copied().collect::<Vec<_>>())
        .set("fill", tile.color(TileLens::Biome).to_html())
        .set(
            "transform",
            format!(
                "translate({x} {y})",
                x = pos.x() as f32 * 1.5,
                y = (pos.x() as f32 / 2.0 + pos.y() as f32) * -(3.0f32.sqrt()),
            ),
        )
        .add(Comment::new(pos.to_string())) // Readability!
}
