import {
  Scene,
  Engine,
  ArcRotateCamera,
  HemisphericLight,
  Vector3,
} from "@babylonjs/core";
import config from "../terra.json";
import WorldRenderer from "./WorldRenderer";
import InputHandler from "./InputHandler";
import PauseMenu from "./PauseMenu";
import { World } from "../wasm";

const { Terra } = await import("../wasm");

/**
 * The scene that handles everything in-game
 */
class WorldScene {
  private scene: Scene;
  private world: World;
  private inputHandler: InputHandler;
  private pauseMenu: PauseMenu;
  private paused: boolean;

  constructor(engine: Engine) {
    // Init scene
    this.scene = new Scene(engine);
    // do a bunch of shit to make it go zoomer fast
    // (doesn't actually make much of a difference)
    this.scene.animationsEnabled = false;
    this.scene.texturesEnabled = false;
    this.scene.proceduralTexturesEnabled = false;
    this.scene.collisionsEnabled = false;
    this.scene.physicsEnabled = false;
    this.scene.fogEnabled = false;
    this.scene.particlesEnabled = false;
    this.scene.blockMaterialDirtyMechanism = true;

    // Init the camera
    const camera = new ArcRotateCamera(
      "camera",
      0,
      Math.PI / 4,
      500.0,
      new Vector3(0.0, 200.0, 0.0),
      this.scene
    );
    camera.lowerRadiusLimit = 1.0;
    camera.upperRadiusLimit = 500.0;
    camera.panningSensibility = 100;
    camera.attachControl(engine.getRenderingCanvas(), true);

    // Init world lighting
    new HemisphericLight("lightSun", new Vector3(0, 1, 0), this.scene);

    this.world = Terra.new_world(config.world);
    const worldRenderer = new WorldRenderer(this.scene, this.world);

    this.scene.freezeActiveMeshes();
    this.scene.freezeMaterials();

    this.inputHandler = new InputHandler(config.input, this, worldRenderer);

    this.scene.onKeyboardObservable.add((kbInfo) =>
      this.inputHandler.handleKeyEvent(kbInfo)
    );

    this.pauseMenu = new PauseMenu();
    this.paused = false;
  }

  setPause(paused: boolean): void {
    this.pauseMenu.setVisible(paused);
  }

  toggleDebugOverlay(): void {
    if (this.scene.debugLayer.isVisible()) {
      this.scene.debugLayer.hide();
    } else {
      this.scene.debugLayer.show();
    }
  }

  render(): void {
    // TODO use paused field here
    this.scene.render();
  }
}

export default WorldScene;
