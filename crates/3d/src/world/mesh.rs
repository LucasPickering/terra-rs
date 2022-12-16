use bevy::{
    prelude::{Mesh, Vec3},
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use terra::{HexDirection, Point2, TilePoint, VertexDirection, WorldRenderer};

#[derive(Copy, Clone, Debug)]
pub struct TileMeshBuilder {
    top: bool,
    /// Ordering matches [HexDirection::CLOCKWISE]
    sides: [bool; 6],
}

impl TileMeshBuilder {
    pub fn disable_sides(mut self) -> Self {
        self.sides = [false; 6];
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
                vertices_2d
                    .iter()
                    .map(|point2| [point2.x as f32, 1.0, point2.y as f32]),
            )
            .collect();

        // Normals are just the vertex vectors, but normalized
        let normals: Vec<Vec3> = positions
            .iter()
            .map(|position| {
                Vec3::new(position[0], 0.0, position[2]).normalize()
            })
            .collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

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

        if self.top {
            indices.extend([
                // Top
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

        mesh.set_indices(Some(Indices::U32(indices)));
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
