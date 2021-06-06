import React, { useState } from "react";
import { Redirect, Route, Switch } from "react-router-dom";
import WorldCanvasWrapper from "./WorldCanvasWrapper";
import DemoContext, { WorldState } from "context/DemoContext";
import type { RenderConfigObject, WorldConfigObject } from "terra-wasm";
import { useConfigHandler } from "hooks/useConfigHandler";
import WorldConfigEditor from "./config/WorldConfigEditor";
const { generate_world, validate_world_config, validate_render_config } =
  await import("terra-wasm");

/**
 * Configure and generate a demo Terra world.
 */
const Demo: React.FC = () => {
  // Use a common handler for each config. This will handle all serialization,
  // validation, deserialization, etc. It will also automatically update the
  // URL when the configs change.
  const worldConfigHandler = useConfigHandler<WorldConfigObject>({
    validator: validate_world_config,
    queryParam: "worldConfig",
  });
  const renderConfigHandler = useConfigHandler<RenderConfigObject>({
    validator: validate_render_config,
    queryParam: "renderConfig",
  });

  const [worldState, setWorldState] = useState<WorldState>({ phase: "empty" });
  const generateWorld = (): void => {
    setWorldState({ phase: "generating" });

    // Update the config query param
    worldConfigHandler.updateQueryParam();

    // Defer world gen to idle time, so the browser prioritizes UI updates
    window.requestIdleCallback(() => {
      try {
        const newWorld = generate_world(worldConfigHandler.config);
        setWorldState({ phase: "populated", world: newWorld });
      } catch (error) {
        // The error will get logged by Rust
        setWorldState({ phase: "error", error });
      }
    });
  };

  return (
    <DemoContext.Provider
      value={{
        worldConfigHandler,
        renderConfigHandler,
        worldState,
        generateWorld,
      }}
    >
      <Switch>
        <Route path="/demo/new" exact>
          <WorldConfigEditor standalone />
        </Route>

        <Route path="/demo/world" exact>
          <WorldCanvasWrapper />
        </Route>

        {/* Redirect everything else to the config page */}
        <Redirect from="*" to="/demo/new" exact />
      </Switch>
    </DemoContext.Provider>
  );
};

export default Demo;
