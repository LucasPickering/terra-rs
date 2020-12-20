import { KeyboardEventTypes, KeyboardInfo, Scene } from "@babylonjs/core";
import { RecursivePartial } from "./util";
import WorldRenderer from "./WorldRenderer";
const { TileLens } = await import("./wasm");

export interface InputConfig {
  /**
   * Input bindings. The values here should correspond to values from
   * the KeyboardEvent.key field.
   * https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent
   */
  bindings: {
    toggleDebugOverlay: string;
    lensComposite: string;
    lensElevation: string;
    lensHumidity: string;
    lensBiome: string;
  };
}

type InputAction = keyof InputConfig["bindings"];

function isInputAction(s: string): s is InputAction {
  return true; // TODO
}

const DEFAULT_INPUT_CONFIG: InputConfig = {
  bindings: {
    toggleDebugOverlay: "`",
    lensComposite: "1",
    lensElevation: "2",
    lensHumidity: "3",
    lensBiome: "4",
  },
};

class InputHandler {
  private config: InputConfig;
  private keyToEvent: Map<string, InputAction>;
  private scene: Scene;
  private worldRenderer: WorldRenderer;

  constructor(
    config: RecursivePartial<InputConfig> | undefined,
    scene: Scene,
    worldRenderer: WorldRenderer
  ) {
    this.config = {
      ...DEFAULT_INPUT_CONFIG,
      ...config,
      bindings: {
        ...DEFAULT_INPUT_CONFIG.bindings,
        ...config?.bindings,
      },
    };
    this.scene = scene;
    this.worldRenderer = worldRenderer;

    this.keyToEvent = new Map();
    Object.entries(this.config.bindings).forEach(([key, value]) => {
      // We could potentially get garbage actions from the user's config, so
      // validate each action here
      if (isInputAction(key)) {
        this.keyToEvent.set(value.toUpperCase(), key);
      } else {
        // eslint-disable-next-line no-console
        console.warn("Unknown input action:", key);
      }
    });
  }

  handleKeyEvent(kbInfo: KeyboardInfo): void {
    if (kbInfo.type === KeyboardEventTypes.KEYDOWN) {
      // Map the keyboard key to a known action
      const action = this.keyToEvent.get(kbInfo.event.key.toUpperCase());
      if (action) {
        this.handleAction(action);
      }
    }
  }

  private handleAction(action: InputAction): void {
    switch (action) {
      case "toggleDebugOverlay":
        if (this.scene.debugLayer.isVisible()) {
          this.scene.debugLayer.hide();
        } else {
          this.scene.debugLayer.show();
        }
        break;
      case "lensComposite":
        this.worldRenderer.updateTileColors(TileLens.Composite);
        break;
      case "lensElevation":
        this.worldRenderer.updateTileColors(TileLens.Elevation);
        break;
      case "lensHumidity":
        this.worldRenderer.updateTileColors(TileLens.Humidity);
        break;
      case "lensBiome":
        this.worldRenderer.updateTileColors(TileLens.Biome);
        break;
    }
  }
}

export default InputHandler;
