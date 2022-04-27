import { ConfigHandler } from "hooks/useConfigHandler";
import React from "react";
import type { WorldConfigObject, World, RenderConfigObject } from "terra-wasm";

export type WorldState =
  | { phase: "empty" }
  | { phase: "generating" }
  | { phase: "error"; error: Error }
  | { phase: "populated"; world: World };

export interface DemoContextType {
  worldConfigHandler: ConfigHandler<WorldConfigObject>;
  renderConfigHandler: ConfigHandler<RenderConfigObject>;
  worldState: WorldState;
  generateWorld: () => void;
}

const DemoContext = React.createContext<DemoContextType>(
  {} as DemoContextType // Safe because this value never gets used
);

export default DemoContext;
