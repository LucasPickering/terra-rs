import {
  Color4,
  InstancedMesh,
  Mesh,
  MeshBuilder,
  Scene,
} from "@babylonjs/core";
import type { TileLens as TileLensType, Tile, World } from "./wasm";
const { TileLens } = await import("./wasm");

const TILE_SIZE = 1.0; // vertex-to-vertex tile diameter

class WorldRenderer {
  private tiles: Array<[Tile, InstancedMesh]>;
  private tileLens: TileLensType;

  constructor(scene: Scene, world: World) {
    const mesh = MeshBuilder.CreateCylinder(
      "tile",
      {
        diameter: TILE_SIZE,
        tessellation: 6,
        cap: Mesh.CAP_END,
      },
      scene
    );
    mesh.convertToUnIndexedMesh();
    mesh.registerInstancedBuffer("color", 4);

    // This call allocates a whole new array, so we store the array instead of
    // the full world object.
    const tiles = world.tiles_array();

    this.tiles = tiles.map((tile) => {
      const pos = tile.pos;
      const name = `tile(${pos.x},${pos.y},${pos.z})`;
      const instance = mesh.createInstance(name);

      // Convert hex coords to pixel coords
      // https://www.redblobgames.com/grids/hexagons/#coordinates-cube
      instance.position.x = pos.x * 0.75 * TILE_SIZE;
      instance.position.z =
        (pos.x / 2 + pos.y) * -(Math.sqrt(3) / 2) * TILE_SIZE;
      instance.position.y = tile.height;
      instance.scaling.y = tile.height;

      instance.freezeWorldMatrix();
      return [tile, instance];
    });

    this.tileLens = TileLens.Composite;
    this.updateTileColors(TileLens.Composite);
  }

  updateTileColors(lens: TileLensType): void {
    this.tileLens = lens;
    this.tiles.forEach(([tile, instance]) => {
      const color = tile.color(this.tileLens);
      instance.instancedBuffers.color = new Color4(
        color.red,
        color.green,
        color.blue,
        1.0
      );
    });
  }
}

export default WorldRenderer;
