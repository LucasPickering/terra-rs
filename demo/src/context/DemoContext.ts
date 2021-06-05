import { ConfigHandler } from "hooks/useConfigHandler";
import React from "react";
import type {
  Terra,
  WorldConfigObject,
  World,
  RenderConfigObject,
} from "terra-wasm";

export interface DemoContextType {
  terra: Terra;
  worldConfigHandler: ConfigHandler<WorldConfigObject>;
  renderConfigHandler: ConfigHandler<RenderConfigObject>;
  generateWorldEnabled: boolean;
  world: World | "generating" | undefined;
  generateWorld: (goToWorld: boolean) => void;
}

const DemoContext = React.createContext<DemoContextType>(
  {} as DemoContextType // Safe because this value never gets used
);

export default DemoContext;
