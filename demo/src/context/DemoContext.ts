import React from "react";
import type { Terra, WorldConfigObject, World } from "terra-wasm";
import { Path } from "../util";

/**
 * Type-safe paths into the config. Pure wizardry ripped from
 * https://stackoverflow.com/a/58436959/1907353
 */
export type ConfigKey = Path<WorldConfigObject>;

export interface DemoContextType {
  terra: Terra;
  config: WorldConfigObject;
  generateWorldEnabled: boolean;
  setConfigValue: (key: ConfigKey, value: unknown) => void;
  resetConfig: () => void; // Reset to default value
  world: World | "generating" | undefined;
  generateWorld: (goToWorld: boolean) => void;
}

const DemoContext = React.createContext<DemoContextType>(
  {} as DemoContextType // Safe because this value never gets used
);

export default DemoContext;
