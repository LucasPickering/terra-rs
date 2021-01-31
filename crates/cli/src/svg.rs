use crate::RenderOptions;
use svg::{
    node::{
        element::{Group, Line, Polygon},
        Comment,
    },
    Document,
};
use terra::{
    Color3, GeoFeature, HasHexPosition, HexAxialDirection, Tile, World,
};

const RIVER_COLOR: Color3 = Color3::new_int(72, 192, 240);

/// Generate an SVG document for a world
pub fn draw_world(world: &World, options: RenderOptions) -> Document {
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
        let polygon = draw_tile(tile, options);
        document = document.add(polygon);
    }

    document
}

/// Generate an SVG polygon for a single tile
fn draw_tile(tile: &Tile, options: RenderOptions) -> Group {
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
                .set("fill", tile.color(options.lens).to_html()),
        );

    // Add overlays for each geo feature
    if options.show_features {
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
