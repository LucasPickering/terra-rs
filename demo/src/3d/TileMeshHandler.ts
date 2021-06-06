import { Matrix, Mesh, MeshBuilder, Scene } from "@babylonjs/core";
import type { Tile, World, RenderConfigObject, Point2 } from "terra-wasm";
const { build_renderer, copy_tiles } = await import("terra-wasm");

/**
 * The length of one side of each tile. This is also the center-to-vertex
 * radius, because each tile is 6 equilateral triangles.
 */
const TILE_SIDE_LENGTH = 1.0;
/**
 * Distance between two opposite vertices.
 */
const TILE_VERTEX_DIAM = TILE_SIDE_LENGTH * 2;

/**
 * Util class for generating and rendering meshes for a world of tiles
 */
class TileMeshHandler {
  private mesh: Mesh;
  /**
   * Each tile in the world paired with the index of its mesh instance.
   */
  private tiles: TileMeshInstance[];

  constructor(scene: Scene, world: World, renderConfig: RenderConfigObject) {
    // We use "thin instances" here for the tiles cause #performance
    // https://doc.babylonjs.com/divingDeeper/mesh/copies/thinInstances
    // There's a section on that page called "Faster thin instances", if we
    // decide that initialization is too slow we could try that

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
    const tiles = copy_tiles(world);

    this.tiles = tiles.map((tile, i) => {
      // Convert hex coords to pixel coords
      // https://www.redblobgames.com/grids/hexagons/#coordinates-cube
      const position2d = tile.pos.to_point2();

      // Refresh meshes if this is the last tile in the list
      const isLastTile = i === tiles.length - 1;
      const instanceIndex = this.mesh.thinInstanceAdd(
        Matrix.Zero(),
        isLastTile
      );

      return { tile, instanceIndex, position2d };
    });

    this.updateRenderConfig(renderConfig);
  }

  /**
   * Re-render the world using the given render config. This will update each
   * tile mesh, without generating any new meshes, so it's fairly cheap.
   */
  updateRenderConfig(renderConfig: RenderConfigObject): void {
    // Build a new renderer with the new config (very cheap)
    const renderer = build_renderer(renderConfig);

    // Update each tile to have the correct color and height
    this.tiles.forEach(({ tile, instanceIndex, position2d }, i) => {
      // Refresh meshes if this is the last tile in the list
      const shouldRefresh = i === this.tiles.length - 1;

      // Update mesh height, in case vertical scaling changed
      const tileHeight = renderer.tile_height(tile);
      // According to the docs, the origin of each tile "cylinder" is the
      // center, so logic tells us that we should have to translate Y by
      // `tileHeight/2` to get all the bottoms to line up. For some reason
      // though, just `tileHeight` is what works ¯\_(ツ)_/¯
      // https://doc.babylonjs.com/divingDeeper/mesh/creation/set/cylinder
      const transformationMatrix = Matrix.Translation(
        position2d.x,
        tileHeight,
        position2d.y
      )
        // Remove previous scaling, then set based on height
        .removeRotationAndScaling()
        .add(Matrix.Scaling(1, tileHeight, 1));
      this.mesh.thinInstanceSetMatrixAt(
        instanceIndex,
        transformationMatrix,
        shouldRefresh // Refresh on last tile
      );

      // Update color, in case tile lens changed
      const color = renderer.tile_color(tile);
      this.mesh.thinInstanceSetAttributeAt(
        "color",
        instanceIndex,
        [color.red, color.green, color.blue, 1.0],
        shouldRefresh // Refresh on last tile
      );
    });
  }
}

interface TileMeshInstance {
  tile: Tile;
  instanceIndex: number;
  position2d: Point2;
}

export default TileMeshHandler;
