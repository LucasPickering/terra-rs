import "@babylonjs/core/Debug/debugLayer";
import "@babylonjs/inspector";
import "@babylonjs/loaders/glTF";
import { Engine } from "@babylonjs/core";
import WorldScene from "./world/WorldScene";

const { Terra } = await import("./wasm");

const CANVAS_ID = "game-canvas";

/**
 * Top-level game class
 */
export class App {
  constructor() {
    const canvas = document.getElementById(
      CANVAS_ID
    ) as HTMLCanvasElement | null;

    if (!canvas) {
      throw new Error(`Could not find canvas by ID: ${CANVAS_ID}`);
    }

    // Initialize Terra once, which will let us generate worlds
    const terra = new Terra();

    // initialize babylon scene and engine
    const engine = new Engine(canvas, false, { audioEngine: false }, true);
    const scene = new WorldScene(terra, engine);

    // run the main render loop
    engine.runRenderLoop(() => {
      scene.render();
    });
  }
}

new App();
