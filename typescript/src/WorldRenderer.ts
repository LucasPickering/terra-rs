import {
  Color4,
  InstancedMesh,
  Mesh,
  MeshBuilder,
  Scene,
} from "@babylonjs/core";
import type { World } from "./wasm";
const { TileLens } = await import("./wasm");

const TILE_SIZE = 1.0; // vertex-to-vertex tile diameter

class WorldRenderer {
  private instances: InstancedMesh[];

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

    const tiles = world.tiles_render_info(TileLens.Composite);
    console.log("tiles", tiles);
    this.instances = tiles.map((tile) => {
      const name = `tile(${tile.x},${tile.y},${tile.z})`;
      const instance = mesh.createInstance(name);

      // Convert hex coords to pixel coords
      // https://www.redblobgames.com/grids/hexagons/#coordinates-cube
      instance.position.x = tile.x * 0.75 * TILE_SIZE;
      instance.position.z =
        (tile.x / 2 + tile.y) * -(Math.sqrt(3) / 2) * TILE_SIZE;
      instance.position.y = tile.height;
      instance.scaling.y = tile.height;

      // Set color
      instance.instancedBuffers.color = new Color4(
        tile.color.red,
        tile.color.green,
        tile.color.blue,
        1.0
      );

      instance.freezeWorldMatrix();
      return instance;
    });
  }

  toggleColor(): void {
    this.instances.forEach((instance) => {
      instance.instancedBuffers.color = new Color4(1.0, 1.0, 1.0, 1.0);
    });
  }
}

export default WorldRenderer;
