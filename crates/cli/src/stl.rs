use crate::RenderOptions;
use stl_io::{Normal, Triangle, Vertex};
use strum::IntoEnumIterator;
use terra::{
    HasHexPosition, HexAxialDirection, HexDirection, HexDirectionMap, Point2,
    Tile, World,
};

pub fn draw_world(world: &World, _options: RenderOptions) -> Vec<Triangle> {
    let mut mesh =
        Vec::with_capacity(world.tiles().len() * TileSolid::TRIANGLES_PER_TILE);

    for tile in world.tiles().values() {
        let solid = TileSolid::new(world, tile);
        solid.add_to_mesh(&mut mesh);
    }

    mesh
}

/// A convenience struct for converting a tile into STL triangles.
#[derive(Clone, Debug)]
struct TileSolid {
    perimeter_points_2d: Vec<Point2>,
    bottom_perimeter_vertices: Vec<Vertex>,
    top_z: f32,
    top_perimeter_vertices: Vec<Vertex>,
    adjacents_z: HexDirectionMap<f32>,
}

impl TileSolid {
    /// AT MOST 20 triangles per tile:
    /// - 4 for the top
    /// - MAXIMUM of 2 per side, times 6 sides
    ///   - These get culled if adjacent to a taller tile
    /// - 4 for the bottom
    const TRIANGLES_PER_TILE: usize = 20;

    /// Vertex indices for the vertices of tile's face (top or bottom). Together
    /// these form the triangles that make up the face. Starts at the top-left
    /// and goes clockwise, the same as [HexAxialDirection::CLOCKWISE].
    const FACE_INDICES: &'static [[usize; 3]] =
        &[[0, 3, 1], [0, 4, 3], [1, 3, 2], [0, 5, 4]];

    fn new(world: &World, tile: &Tile) -> Self {
        let center_2d = tile.position().to_point2();
        let perimeter_points_2d: Vec<_> = HexAxialDirection::CLOCKWISE
            .iter()
            .cloned()
            .map(|dir| center_2d + dir.to_vector2())
            .collect();

        let bottom_perimeter_vertices: Vec<_> = perimeter_points_2d
            .iter()
            .map(|p| Vertex::new([p.x as f32, p.y as f32, 0.0]))
            .collect();

        let top_z = tile.height().0 as f32;
        let top_perimeter_vertices: Vec<_> = perimeter_points_2d
            .iter()
            .map(|p| Vertex::new([p.x as f32, p.y as f32, top_z]))
            .collect();

        let pos = tile.position();
        let tiles = world.tiles();
        let adjacents_z = HexDirection::iter()
            .filter_map(|dir| {
                let adj_pos = pos + dir.to_vector();
                let adj_tile = tiles.get(&adj_pos)?;
                Some((dir, adj_tile.height().0 as f32))
            })
            .collect();

        Self {
            perimeter_points_2d,
            bottom_perimeter_vertices,
            top_z,
            top_perimeter_vertices,
            adjacents_z,
        }
    }

    /// Convert this tile to triangle soup and add them to the soup pot.
    fn add_to_mesh(self, mesh: &mut Vec<Triangle>) {
        // Normals are bullshit anyway, most programs don't respect them
        let normal = Normal::new([0.0, 0.0, 0.0]);

        // REMEMBER: We use the right-hand rule, so all vertices are
        // COUNTER-CLOCKWISE when looking at the visible side.

        // Bottom face
        for [i1, i2, i3] in Self::FACE_INDICES {
            let vertices = [
                self.bottom_perimeter_vertices[*i1],
                self.bottom_perimeter_vertices[*i2],
                self.bottom_perimeter_vertices[*i3],
            ];
            mesh.push(Triangle { normal, vertices })
        }

        for [i1, i2, i3] in Self::FACE_INDICES {
            let vertices = [
                // Reverse these so the hex faces up, not down
                self.top_perimeter_vertices[*i3],
                self.top_perimeter_vertices[*i2],
                self.top_perimeter_vertices[*i1],
            ];
            mesh.push(Triangle { normal, vertices })
        }

        // For each side of the hexagon, draw 2 triangles
        for (i, dir) in HexDirection::iter().enumerate() {
            let bottom_z = *self.adjacents_z.get(&dir).unwrap_or(&0.0);

            // If the adjacent tile in this direction is taller, then no need
            // to draw a side here because it won't be visible.
            if bottom_z <= self.top_z {
                // Some setup variables
                let i1 = i;
                let i2 = (i + 1) % 6;
                let p1 = self.perimeter_points_2d[i1];
                let p2 = self.perimeter_points_2d[i2];

                let bottom_v1 =
                    Vertex::new([p1.x as f32, p1.y as f32, bottom_z]);
                let bottom_v2 =
                    Vertex::new([p2.x as f32, p2.y as f32, bottom_z]);

                let top_v1 = self.top_perimeter_vertices[i1];
                let top_v2 = self.top_perimeter_vertices[i2];

                // Side face - here's what each one looks like from the OUTSIDE
                // We slice it into two triangles for our triangle soup
                //
                //    top_v1       top_v2
                //          +-----+
                //          |\    |
                //          | \   |
                //          |  \  |
                //          |   \ |
                //          |    \|
                //          +-----+
                // bottom_v1       bottom_v2

                // Remember, all vertices are CCW (right-hand rule)
                // Top-right triangle
                mesh.push(Triangle {
                    normal,
                    vertices: [bottom_v2, top_v2, top_v1],
                });
                // Bottom-left triangle
                mesh.push(Triangle {
                    normal,
                    vertices: [top_v1, bottom_v1, bottom_v2],
                });
            }
        }
    }
}
