import "@babylonjs/core/Debug/debugLayer";
import "@babylonjs/inspector";
import "@babylonjs/loaders/glTF";
import {
  Scene,
  Engine,
  ArcRotateCamera,
  HemisphericLight,
  MeshBuilder,
  Vector3,
  Mesh,
  Color4,
} from "@babylonjs/core";
import config from "./terra.json";
import type { TileRenderInfo } from "./wasm";

const { Terra } = await import("./wasm");

const CANVAS_ID = "game-canvas";
const TILE_SIZE = 1.0; // vertex-to-vertex tile diameter

function makeTileMeshes(
  engine: Engine,
  scene: Scene,
  tiles: TileRenderInfo[]
): void {
  const color = new Color4(1.0, 0.0, 0.0, 1.0);
  const mesh = MeshBuilder.CreateCylinder(
    "tile",
    {
      diameter: TILE_SIZE,
      tessellation: 6,
      cap: Mesh.CAP_END,
      faceColors: [color, color, color, color, color, color, color],
    },
    scene
  );
  mesh.convertToUnIndexedMesh();
  mesh.registerInstancedBuffer("color", 4);

  tiles.forEach((tile) => {
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
  });
}

class App {
  constructor() {
    const canvas = document.getElementById(
      CANVAS_ID
    ) as HTMLCanvasElement | null;

    if (!canvas) {
      throw new Error(`Could not find canvas by ID: ${CANVAS_ID}`);
    }

    // initialize babylon scene and engine
    const engine = new Engine(canvas, false, { audioEngine: false }, true);
    const scene = new Scene(engine);
    // do a bunch of shit to make it go zoomer fast
    scene.animationsEnabled = false;
    scene.texturesEnabled = false;
    scene.proceduralTexturesEnabled = false;
    scene.collisionsEnabled = false;
    scene.physicsEnabled = false;
    scene.fogEnabled = false;
    scene.particlesEnabled = false;
    scene.blockMaterialDirtyMechanism = true;

    // Init the camera
    const camera = new ArcRotateCamera(
      "Camera",
      Math.PI / 2,
      Math.PI / 2,
      2.0,
      new Vector3(0.0, 210.0, 0.0),
      scene
    );
    camera.lowerRadiusLimit = 1.0;
    camera.upperRadiusLimit = 100.0;
    camera.panningSensibility = 100;
    camera.attachControl(canvas, true);

    // Init world lighting
    new HemisphericLight("lightSun", new Vector3(0, 1, 0), scene);

    const world = Terra.new_world(config.world);
    const tiles = world.tiles_array();
    makeTileMeshes(engine, scene, tiles);
    scene.freezeActiveMeshes();
    scene.freezeMaterials();

    // hide/show the Inspector
    window.addEventListener("keydown", (ev) => {
      // Shift+Ctrl+Alt+I
      if (ev.shiftKey && ev.ctrlKey && ev.altKey && ev.keyCode === 73) {
        if (scene.debugLayer.isVisible()) {
          scene.debugLayer.hide();
        } else {
          scene.debugLayer.show();
        }
      }
    });

    // run the main render loop
    engine.runRenderLoop(() => {
      scene.render();
    });
  }
}
new App();
