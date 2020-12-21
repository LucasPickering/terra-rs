import "@babylonjs/core/Debug/debugLayer";
import "@babylonjs/inspector";
import "@babylonjs/loaders/glTF";
import { Engine } from "@babylonjs/core";
import WorldScene from "./world/WorldScene";

const CANVAS_ID = "game-canvas";

/**
 * Top-level game class
 */
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
    const scene = new WorldScene(engine);

    // run the main render loop
    engine.runRenderLoop(() => {
      scene.render();
    });
  }
}

new App();
