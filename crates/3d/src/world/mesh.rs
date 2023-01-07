use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
};
use terra::{
    HasHexPosition, HexDirection, Point2, Tile, TileDirection, TilePoint,
    VertexDirection, World, WorldRenderer,
};

pub fn build_mesh(world: &World, renderer: &WorldRenderer) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut vertices: Vec<[f32; 3]> =
    // TODO fix capacity, since vertices get duplicated
        Vec::with_capacity(world.tiles().len() * 6);
    for tile in world.tiles().values() {
        let height = renderer.tile_height(tile);

        // Helper to convert a 2D point to a 3D one that bevy will takea

        let center_vertex = to_vertex(renderer.tile_position(tile), height);
        let top_vertices: HashMap<_, _> = VertexDirection::CLOCKWISE
            .iter()
            .map(|&direction| {
                (
                    direction,
                    to_vertex(
                        renderer.tile_vertex_position(tile, direction),
                        height,
                    ),
                )
            })
            .collect();

        // For each side of the hexagon:
        // TODO comment
        // TODO optimize to be 4 triangles instead of 6
        for &side in TileDirection::CLOCKWISE {
            let (left_direction, right_direction) =
                side.adjacent_vertex_directions();
            // Get the vertices on either end of this side. Perspective is
            // looking down at the tile from above.
            let left_vertex = top_vertices[&left_direction];
            let right_vertex = top_vertices[&right_direction];

            // TODO is this ordering correct?
            vertices.extend([left_vertex, right_vertex, center_vertex]);

            let neighbor_elevation = world
                .get_adjacent(tile.position(), side)
                .map(Tile::elevation)
                .unwrap_or(World::ELEVATION_RANGE.min);
            // If our neighbor is lower than us (or if this is the edge of the
            // world), then we want to draw the vertical side here too
            if tile.elevation() > neighbor_elevation {
                let neighbor_height =
                    renderer.elevation_to_height(neighbor_elevation);
                let bottom_left_vertex = to_vertex(
                    renderer.tile_vertex_position(tile, left_direction),
                    neighbor_height,
                );
                let bottom_right_vertex = to_vertex(
                    renderer.tile_vertex_position(tile, right_direction),
                    neighbor_height,
                );
                // Add two triangles here to form the rectangular side
                vertices.extend([
                    // Left side
                    bottom_right_vertex,
                    right_vertex,
                    left_vertex,
                    // Right side
                    bottom_left_vertex,
                    bottom_right_vertex,
                    left_vertex,
                ]);
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.compute_flat_normals();
    mesh
}

/// Convert a 2D point + height to a 3D point that we can give to bevy
fn to_vertex(point: Point2, height: f64) -> [f32; 3] {
    [point.x as f32, height as f32, point.y as f32]
}

#[derive(Copy, Clone, Debug)]
pub struct TileMeshBuilder {
    top: bool,
    /// Ordering matches [HexDirection::CLOCKWISE]
    sides: [bool; 6],
}

impl TileMeshBuilder {
    pub fn water(mut self) -> Self {
        self.sides = [false; 6];
        self.top = false;
        self
    }

    pub fn build(&self, renderer: &WorldRenderer) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // A tile has 12 vertices, 6 on top and 6 on bottom. In this order:
        // Bot-N, Bot-ENE, Bot-ESE, Bot-S, Bot-WSW, Bot-WNW
        // Top-N, Top-ENE, Top-ESE, Top-S, Top-WSW, Top-WNW
        let vertices_2d: Vec<Point2> = VertexDirection::CLOCKWISE
            .iter()
            .copied()
            .map(|direction| {
                renderer
                    .hex_to_screen_space(TilePoint::ORIGIN.vertex(direction))
            })
            .collect();
        let positions: Vec<[f32; 3]> = vertices_2d
            .iter()
            // Bottom 6
            .map(|point2| [point2.x as f32, 0.0, point2.y as f32])
            // Top 6
            .chain(
                self.top
                    .then(|| {
                        vertices_2d.iter().map(|point2| {
                            [point2.x as f32, 1.0, point2.y as f32]
                        })
                    })
                    .into_iter()
                    .flatten(),
            )
            .collect();

        // REMEMBER: all vertices are specified CLOCKWISE

        //   Bottom
        //      0
        //     / \
        //    /   \
        //   / T1  \
        //  /       \
        // 5.........1
        // |       ..|
        // | T2  ..  |
        // |   .. T3 |
        // | ..      |
        // 4.........2
        //  \       /
        //   \  T4 /
        //    \   /
        //     \ /
        //      3

        //   Top
        //      6
        //     / \
        //    /   \
        //   / T1  \
        //  /       \
        // 11........7
        // |       ..|
        // | T2  ..  |
        // |   .. T3 |
        // | ..      |
        // 10........8
        //  \       /
        //   \  T4 /
        //    \   /
        //     \ /
        //      9

        // A tile is made up of 16 polygons: 2 per side plus 4 on top
        // We *skip* the bottom because it's not visible anyway
        // Each polygon is 3 vertices
        let mut indices: Vec<u32> = Vec::new();
        let mut indexes_used: Vec<u32> = Vec::new();

        indexes_used.extend((0..6).into_iter());
        indices.extend([
            0, 1, 5, // T1
            1, 4, 5, // T2
            1, 2, 4, // T3
            2, 3, 4, // T4
        ]);

        if self.top {
            indexes_used.extend((6..12).into_iter());
            indices.extend([
                6, 7, 11, // T1
                7, 10, 11, // T2
                7, 8, 10, // T3
                8, 9, 10, // T4
            ]);
        }

        // For each side of the hexagon, draw 2 triangles
        for i in 0..6 {
            if self.sides[i] {
                // The side has 4 vertices
                let bottom_right = i as u32;
                let bottom_left = (bottom_right + 1) % 6;
                let top_right = bottom_right + 6;
                let top_left = bottom_left + 6;
                // Split the rectangle into two triangles
                indices.extend([
                    // Bottom-right triangle
                    bottom_right,
                    bottom_left,
                    top_right,
                    // Top-left triangle
                    bottom_left,
                    top_left,
                    top_right,
                ]);
            }
        }

        let positions_used: Vec<[f32; 3]> = indexes_used
            .iter()
            .map(|i| positions[*i as usize])
            .collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions_used);
        mesh.set_indices(Some(Indices::U32(indices)));

        mesh.duplicate_vertices();
        mesh.compute_flat_normals();

        mesh
    }
}

impl Default for TileMeshBuilder {
    fn default() -> Self {
        Self {
            top: true,
            sides: [true; 6],
        }
    }
}
