import { Matrix, Mesh, MeshBuilder, Scene } from "@babylonjs/core";
import type {
  Tile,
  World,
  Terra,
  WorldRenderer,
  RenderConfigObject,
} from "terra-wasm";

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
 * Util class for rendering a world of tiles
 */
class WorldRenderManager {
  private mesh: Mesh;
  /**
   * Each tile in the world paired with the index of its mesh instance.
   */
  private tiles: Array<[Tile, number]>;
  private renderer: WorldRenderer;
  private renderConfig: RenderConfigObject;

  constructor(private readonly terra: Terra, scene: Scene, world: World) {
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

    this.renderConfig = this.terra.default_render_config();
    this.renderer = this.terra.build_renderer(
      this.terra.deserialize_render_config(this.renderConfig)
    );

    // This call allocates a whole new array, so we store the array instead of
    // the full world object.
    const tiles = this.terra.copy_tiles(world);

    this.tiles = tiles.map((tile, i) => {
      // Convert hex coords to pixel coords
      // https://www.redblobgames.com/grids/hexagons/#coordinates-cube
      const pos2d = tile.pos.to_point2();
      const tileHeight = this.renderer.tile_height(tile);
      const transformMatrix = Matrix.Translation(
        pos2d.x,
        tileHeight,
        pos2d.y
        // I'm not entirely sure why this scaling works, but it does
      ).add(Matrix.Scaling(0, tileHeight, 0));

      // Refresh meshes if this is the last tile in the list
      const isLastTile = i === tiles.length - 1;
      const idx = this.mesh.thinInstanceAdd(transformMatrix, isLastTile);

      return [tile, idx];
    });

    this.updateTileColors(this.renderConfig.tile_lens);
  }

  updateTileColors(lens: RenderConfigObject["tile_lens"]): void {
    // Update the tile lens in the config, and build a new renderer
    // Renderers are super cheap to build so this will be zoomer fast
    this.renderConfig.tile_lens = lens;
    this.renderer = this.terra.build_renderer(
      this.terra.deserialize_render_config(this.renderConfig)
    );

    this.tiles.forEach(([tile, instanceIdx], i) => {
      const isLastTile = i === this.tiles.length - 1;
      const color = this.renderer.tile_color(tile);
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

export default WorldRenderManager;
