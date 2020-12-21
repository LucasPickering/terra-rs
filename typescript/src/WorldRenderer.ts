import { Matrix, Mesh, MeshBuilder, Scene } from "@babylonjs/core";
import type { TileLens as TileLensType, Tile, World } from "./wasm";
const { TileLens } = await import("./wasm");

/**
 * The length of one side of each tile. This is also the center-to-vertex
 * radius, because each tile is 6 equilateral triangles.
 */
const TILE_SIDE_LENGTH = 1.0;
/**
 * Distance between two opposite vertices.
 */
const TILE_VERTEX_DIAM = TILE_SIDE_LENGTH * 2;

class WorldRenderer {
  private mesh: Mesh;
  /**
   * Each tile in the world paired with the index of its mesh instance.
   */
  private tiles: Array<[Tile, number]>;
  private tileLens: TileLensType;

  constructor(scene: Scene, world: World) {
    // We use "thin instances" here for the tiles cause #performance
    // https://doc.babylonjs.com/divingDeeper/mesh/copies/thinInstances
    // TODO there's a section on that page called "Faster thin instances", use
    // that to speed up initialization

    this.mesh = MeshBuilder.CreateCylinder(
      "tile",
      {
        diameter: TILE_VERTEX_DIAM,
        tessellation: 6,
        cap: Mesh.CAP_END,
      },
      scene
    );
    this.mesh.convertToUnIndexedMesh();
    this.mesh.thinInstanceRegisterAttribute("color", 4);

    // This call allocates a whole new array, so we store the array instead of
    // the full world object.
    const tiles = world.tiles_array();

    this.tiles = tiles.map((tile, i) => {
      // Convert hex coords to pixel coords
      // https://www.redblobgames.com/grids/hexagons/#coordinates-cube
      const pos = tile.pos;
      const transformMatrix = Matrix.Translation(
        pos.x * 0.75 * TILE_VERTEX_DIAM,
        tile.height,
        (pos.x / 2 + pos.y) * -(Math.sqrt(3) / 2) * TILE_VERTEX_DIAM
        // I'm not entirely sure why this scaling works, but it does
      ).add(Matrix.Scaling(0, tile.height, 0));

      // Refresh meshes if this is the last tile in the list
      const isLastTile = i === tiles.length - 1;
      const idx = this.mesh.thinInstanceAdd(transformMatrix, isLastTile);

      return [tile, idx];
    });

    this.tileLens = TileLens.Biome;
    this.updateTileColors(this.tileLens);
  }

  updateTileColors(lens: TileLensType): void {
    this.tileLens = lens;
    this.tiles.forEach(([tile, instanceIdx], i) => {
      const isLastTile = i === this.tiles.length - 1;
      const color = tile.color(this.tileLens);
      this.mesh.thinInstanceSetAttributeAt(
        "color",
        instanceIdx,
        [color.red, color.green, color.blue, 1.0],
        // Refresh meshes if this is the last tile in the list
        isLastTile
      );
    });
  }
}

export default WorldRenderer;
