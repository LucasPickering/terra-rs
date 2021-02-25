import {
  Scene,
  Engine,
  ArcRotateCamera,
  HemisphericLight,
  Vector3,
} from "@babylonjs/core";
import WorldRenderer from "./WorldRenderer";
import InputHandler from "./InputHandler";
import type { Terra, TileLens, WasmWorld } from "terra-wasm";
import { hexCodeToColor4 } from "../util";
import theme from "../theme";

export interface NoiseFnConfig {
  octaves: number;
  frequency: number;
  lacunarity: number;
  persistence: number;
}

function initScene(engine: Engine): Scene {
  // Init world scene
  const scene = new Scene(engine);
  // consistency!
  scene.clearColor = hexCodeToColor4(theme().palette.background.default);
  // do a bunch of shit to make it go zoomer fast
  // (doesn't actually make much of a difference)
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
    "camera",
    0,
    Math.PI / 4,
    500.0,
    new Vector3(0.0, 200.0, 0.0),
    scene
  );
  camera.lowerRadiusLimit = 1.0;
  camera.upperRadiusLimit = 500.0;
  camera.panningSensibility = 100;
  camera.attachControl(engine.getRenderingCanvas(), true);

  // Init world lighting
  new HemisphericLight("lightSun", new Vector3(0, 1, 0), scene);

  return scene;
}

/**
 * The scene that handles everything in-game. Config is an unknown type so we
 * can let Rust handle validation etc.
 */
class WorldScene {
  private inputHandler: InputHandler;
  private scene: Scene;
  private worldRenderer: WorldRenderer;

  constructor(terra: Terra, engine: Engine, world: WasmWorld) {
    // Init world scene
    this.scene = initScene(engine);

    this.worldRenderer = new WorldRenderer(terra, this.scene, world);
    this.scene.freezeActiveMeshes();

    this.inputHandler = new InputHandler(this);

    this.scene.onKeyboardObservable.add((kbInfo) =>
      this.inputHandler.handleKeyEvent(kbInfo)
    );
  }

  toggleDebugOverlay(): void {
    if (this.scene.debugLayer.isVisible()) {
      this.scene.debugLayer.hide();
    } else {
      this.scene.debugLayer.show();
    }
  }

  setTileLens(lens: TileLens): void {
    this.worldRenderer.updateTileColors(lens);
  }

  render(): void {
    this.scene.render();
  }
}

export default WorldScene;
