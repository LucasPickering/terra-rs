import "@babylonjs/core/Debug/debugLayer";
import "@babylonjs/loaders/glTF";
import { Engine } from "@babylonjs/core";
import WorldScene from "./WorldScene";
import type { Terra, World } from "terra-wasm";
import { debounce } from "../util";

// This dependency is huge so only pull it in for dev
if (process.env.NODE_ENV === "development") {
  import("@babylonjs/inspector");
}

/**
 * First level of the babylon.js world
 */
export default class WorldDemo {
  constructor(canvas: HTMLCanvasElement, terra: Terra, world: World) {
    // initialize babylon scene and engine
    const engine = new Engine(canvas, true, { audioEngine: false }, false);

    // TODO this doesn't fully work, we need to resize the camera too after
    // an engine resize
    const resizeEngine = debounce(engine.resize, 1000);
    window.onresize = () => {
      resizeEngine();
    };

    const scene = new WorldScene(terra, engine, world);

    // run the main render loop
    engine.runRenderLoop(() => {
      scene.render();
    });
  }
}
