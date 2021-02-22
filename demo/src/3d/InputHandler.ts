import { KeyboardEventTypes, KeyboardInfo } from "@babylonjs/core";
import { assertUnreachable } from "../util";
import WorldScene from "./WorldScene";
const { TileLens } = await import("terra-wasm");

const INPUT_ACTIONS = [
  "toggleDebugOverlay",
  "lensSurface",
  "lensBiome",
  "lensElevation",
  "lensHumidity",
  "lensRunoff",
] as const;
type InputAction = typeof INPUT_ACTIONS[number];

export interface InputConfig {
  /**
   * Input bindings. The values here should correspond to values from
   * the KeyboardEvent.key field.
   * https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent
   */
  bindings: Record<InputAction, string>;
}

function isInputAction(s: string): s is InputAction {
  return (INPUT_ACTIONS as readonly string[]).includes(s);
}

const DEFAULT_INPUT_CONFIG: InputConfig = {
  bindings: {
    toggleDebugOverlay: "`",
    lensSurface: "1",
    lensBiome: "2",
    lensElevation: "3",
    lensHumidity: "4",
    lensRunoff: "5",
  },
};

class InputHandler {
  private config: InputConfig;
  private keyToEvent: Map<string, InputAction>;
  private scene: WorldScene;

  constructor(scene: WorldScene) {
    this.config = DEFAULT_INPUT_CONFIG;
    this.scene = scene;

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
        this.scene.toggleDebugOverlay();
        break;
      case "lensSurface":
        this.scene.setTileLens(TileLens.Surface);
        break;
      case "lensBiome":
        this.scene.setTileLens(TileLens.Biome);
        break;
      case "lensElevation":
        this.scene.setTileLens(TileLens.Elevation);
        break;
      case "lensHumidity":
        this.scene.setTileLens(TileLens.Humidity);
        break;
      case "lensRunoff":
        this.scene.setTileLens(TileLens.Runoff);
        break;
      // Make sure this switch is exhaustive
      default:
        assertUnreachable(action);
    }
  }
}

export default InputHandler;
