import { AdvancedDynamicTexture, Button, Rectangle } from "@babylonjs/gui";

/**
 * In-game pause menu
 */
class PauseMenu {
  private texture: AdvancedDynamicTexture;
  private container: Rectangle;

  constructor() {
    this.texture = AdvancedDynamicTexture.CreateFullscreenUI(
      "worldGenMenu",
      false
    );

    // Add a container for the menu so we can toggle its visible easily
    this.container = new Rectangle();
    this.container.isVisible = false;
    this.texture.addControl(this.container);

    const button = Button.CreateSimpleButton("button1", "Click My Ass");
    this.container.addControl(button);
  }

  setVisible(visible: boolean): void {
    this.container.isVisible = visible;
  }
}

export default PauseMenu;
