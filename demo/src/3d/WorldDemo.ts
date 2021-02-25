import "@babylonjs/core/Debug/debugLayer";
import "@babylonjs/loaders/glTF";
import { Engine } from "@babylonjs/core";
import WorldScene from "./WorldScene";
import type { Terra, WasmWorld } from "terra-wasm";
import { debounce } from "../util";

// This dependency is huge so only pull it in for dev
if (process.env.NODE_ENV === "development") {
  import("@babylonjs/inspector");
}

/**
 * First level of the babylon.js world
 */
export default class WorldDemo {
  private readonly engine: Engine;

  constructor(canvas: HTMLCanvasElement, terra: Terra, world: WasmWorld) {
    // initialize babylon scene and engine
    this.engine = new Engine(canvas, true, { audioEngine: false }, false);

    // TODO this doesn't fully work, we need to resize the camera too after
    // an engine resize
    const resizeEngine = debounce(this.engine.resize, 1000);
    window.onresize = () => {
      resizeEngine();
    };

    const scene = new WorldScene(terra, this.engine, world);

    // run the main render loop
    this.engine.runRenderLoop(() => {
      scene.render();
    });
  }

  /**
   * Delete all babylon resources
   */
  dispose(): void {
    this.engine.dispose();
  }
}
