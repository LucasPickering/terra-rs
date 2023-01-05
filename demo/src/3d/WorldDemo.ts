import "@babylonjs/core/Debug/debugLayer";
import "@babylonjs/loaders/glTF";
import { Engine } from "@babylonjs/core";
import WorldScene from "./WorldScene";
import type { RenderConfigObject, World } from "terra";
import { debounce } from "../util";

// This dependency is huge so only pull it in for dev
if (process.env.NODE_ENV === "development") {
  import("@babylonjs/inspector");
}

/**
 * First level of the babylon.js world
 */
class WorldDemo {
  private readonly engine: Engine;
  private readonly scene: WorldScene;

  constructor(
    canvas: HTMLCanvasElement,
    world: World,
    renderConfig: RenderConfigObject
  ) {
    // initialize babylon scene and engine
    this.engine = new Engine(canvas, true, { audioEngine: false }, false);

    const resizeEngine = debounce(() => this.engine.resize(), 1000);
    window.onresize = () => {
      resizeEngine();
    };

    this.scene = new WorldScene(this.engine, world, renderConfig);

    // run the main render loop
    this.engine.runRenderLoop(() => {
      this.scene.render();
    });
  }

  updateRenderConfig(renderConfig: RenderConfigObject): void {
    this.scene.updateRenderConfig(renderConfig);
  }

  /**
   * Delete all babylon resources
   */
  dispose(): void {
    this.engine.dispose();
  }
}

export default WorldDemo;
