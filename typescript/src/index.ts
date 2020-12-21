import "@babylonjs/core/Debug/debugLayer";
import "@babylonjs/inspector";
import "@babylonjs/loaders/glTF";
import {
  Scene,
  Engine,
  ArcRotateCamera,
  HemisphericLight,
  Vector3,
} from "@babylonjs/core";
import config from "./terra.json";
import WorldRenderer from "./WorldRenderer";
import InputHandler from "./InputHandler";

const { Terra } = await import("./wasm");

const CANVAS_ID = "game-canvas";

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
      new Vector3(0.0, 200.0, 0.0),
      scene
    );
    camera.lowerRadiusLimit = 1.0;
    camera.upperRadiusLimit = 500.0;
    camera.panningSensibility = 100;
    camera.attachControl(canvas, true);

    // Init world lighting
    new HemisphericLight("lightSun", new Vector3(0, 1, 0), scene);

    const world = Terra.new_world(config.world);
    const worldRenderer = new WorldRenderer(scene, world);

    scene.freezeActiveMeshes();
    scene.freezeMaterials();

    const inputHandler = new InputHandler(config.input, scene, worldRenderer);

    scene.onKeyboardObservable.add((kbInfo) =>
      inputHandler.handleKeyEvent(kbInfo)
    );

    // run the main render loop
    engine.runRenderLoop(() => {
      scene.render();
    });
  }
}

new App();
