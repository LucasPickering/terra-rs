import { Engine, Scene, UniversalCamera, Vector3 } from "@babylonjs/core";
import { AdvancedDynamicTexture, XmlLoader } from "@babylonjs/gui";
import WorldScene from "./WorldScene";

/**
 * In-game pause menu
 */
class PauseMenu {
  private scene: Scene;
  private texture: AdvancedDynamicTexture;

  constructor(engine: Engine, worldScene: WorldScene) {
    this.scene = new Scene(engine);
    new UniversalCamera("camera", new Vector3(0, 0, 0), this.scene);
    this.texture = AdvancedDynamicTexture.CreateFullscreenUI(
      "worldGenMenu",
      true,
      this.scene
    );

    const xmlLoader = new XmlLoader();
    xmlLoader.loadLayout("/gui/pause.xml", this.texture, () => {
      xmlLoader.getNodeById("unpause").onPointerUpObservable.add(() => {
        worldScene.setPaused(false);
      });
    });
  }

  render(): void {
    this.scene.render();
  }
}

export default PauseMenu;
