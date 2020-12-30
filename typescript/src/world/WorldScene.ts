import {
  Scene,
  Engine,
  ArcRotateCamera,
  HemisphericLight,
  Vector3,
} from "@babylonjs/core";
import WorldRenderer from "./WorldRenderer";
import InputHandler from "./InputHandler";
import type { Terra, TileLens, World } from "../wasm";
import PauseMenu from "./PauseMenu";

// We'll let Rust enforce the correct type here
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const config: any = await import("../world.json");

export interface NoiseFnConfig {
  octaves: number;
  frequency: number;
  lacunarity: number;
  persistence: number;
}

function initScene(engine: Engine): Scene {
  // Init world scene
  const scene = new Scene(engine);
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

function getWorldConfig(): Record<string, unknown> {
  const queryParams = new URLSearchParams(window.location.search);

  const cfg = {
    // bullshit here to pick a random seed if we don't have one
    seed: Math.round(Math.random() * Number.MAX_SAFE_INTEGER),
    ...config,
  };

  // This is shitty but it works for now
  const addQueryParam = (param: string): void => {
    const val = queryParams.get(param);
    const parsed = parseInt(val ?? "", 10);
    if (Number.isFinite(parsed)) {
      cfg[param] = parsed;
    }
  };

  addQueryParam("seed");
  addQueryParam("radius");

  return cfg;
}

/**
 * The scene that handles everything in-game
 */
class WorldScene {
  private terra: Terra;
  private inputHandler: InputHandler;
  private scene: Scene;
  private pauseMenu: PauseMenu;
  private paused: boolean;
  private world: World;
  private worldRenderer: WorldRenderer;

  constructor(terra: Terra, engine: Engine) {
    this.terra = terra;
    // Init world scene
    this.scene = initScene(engine);

    // Generate the world
    this.world = this.terra.generate_world(getWorldConfig());
    this.worldRenderer = new WorldRenderer(this.scene, this.world);
    this.scene.freezeActiveMeshes();

    this.inputHandler = new InputHandler(this);

    this.scene.onKeyboardObservable.add((kbInfo) =>
      this.inputHandler.handleKeyEvent(kbInfo)
    );

    // Init pause menu
    this.pauseMenu = new PauseMenu(engine, this);
    this.paused = false;
  }

  setPaused(paused: boolean): void {
    this.paused = paused;
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
    if (this.paused) {
      this.pauseMenu.render();
    } else {
      this.scene.render();
    }
  }
}

export default WorldScene;
