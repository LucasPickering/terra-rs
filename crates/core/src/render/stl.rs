//! This module provides logic for rendering a world as an STL. Only available
//! with the "stl" feature enabled.

use crate::{
    render::{unit::Point2, WorldRenderer},
    world::hex::{TileDirectionMap, VertexDirection},
    HasHexPosition, HexDirection, Tile, TileDirection, World,
};
use stl_io::{Normal, Triangle, Vertex};
use strum::IntoEnumIterator;

/// Render the given world as an STL model. STL only carries geometric data,
/// so no colors/textures. There's no dominant convention around which axis
/// should be up in an STL, so here we consider the **Y axis to be up and
/// down** to be consistent with the demo.
pub fn world_to_stl(world: &World, renderer: &WorldRenderer) -> Vec<Triangle> {
    let tiles = world.tiles();
    let mut mesh =
        Vec::with_capacity(tiles.len() * TileSolid::TRIANGLES_PER_TILE);

    for tile in tiles.values() {
        let solid = TileSolid::new(world, renderer, tile);
        solid.add_to_mesh(&mut mesh);
    }

    mesh
}

/// A convenience struct for converting a tile into STL triangles.
#[derive(Clone, Debug)]
struct TileSolid {
    perimeter_points_2d: Vec<Point2>,
    bottom_perimeter_vertices: Vec<Vertex>,
    top_y: f32,
    top_perimeter_vertices: Vec<Vertex>,
    adjacents_y: TileDirectionMap<f32>,
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

    fn new(world: &World, renderer: &WorldRenderer, tile: &Tile) -> Self {
        let perimeter_points_2d: Vec<Point2> = VertexDirection::CLOCKWISE
            .iter()
            .cloned()
            // Get a 2D point for each hexagon vertex
            .map(|dir| {
                renderer.hex_to_screen_space(tile.position().vertex(dir))
            })
            .collect();

        let bottom_perimeter_vertices: Vec<_> = perimeter_points_2d
            .iter()
            .map(|p| Vertex::new([p.x as f32, 0.0, p.y as f32]))
            .collect();

        let top_y = renderer.tile_height(tile) as f32;
        let top_perimeter_vertices: Vec<_> = perimeter_points_2d
            .iter()
            .map(|p| Vertex::new([p.x as f32, top_y, p.y as f32]))
            .collect();

        let pos = tile.position();
        let adjacents_y = TileDirection::iter()
            .filter_map(|dir| {
                let adj_pos = pos.adjacent(dir);
                let adj_tile = world.tiles().get(&adj_pos)?;
                Some((dir, renderer.tile_height(adj_tile) as f32))
            })
            .collect();

        Self {
            perimeter_points_2d,
            bottom_perimeter_vertices,
            top_y,
            top_perimeter_vertices,
            adjacents_y,
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
        for (i, dir) in TileDirection::iter().enumerate() {
            let bottom_y = *self.adjacents_y.get(&dir).unwrap_or(&0.0);

            // If the adjacent tile in this direction is taller, then no need
            // to draw a side here because it won't be visible.
            if bottom_y <= self.top_y {
                // Some setup variables
                let i1 = i;
                let i2 = (i + 1) % 6;
                let p1 = self.perimeter_points_2d[i1];
                let p2 = self.perimeter_points_2d[i2];

                let bottom_v1 =
                    Vertex::new([p1.x as f32, bottom_y, p1.y as f32]);
                let bottom_v2 =
                    Vertex::new([p2.x as f32, bottom_y, p2.y as f32]);

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
