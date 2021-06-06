import React, { useState } from "react";
import { Redirect, Route, Switch } from "react-router-dom";
import WorldCanvasWrapper from "./WorldCanvasWrapper";
import DemoContext from "context/DemoContext";
import type { RenderConfigObject, World, WorldConfigObject } from "terra-wasm";
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

  const [world, setWorld] = useState<World | "generating" | undefined>();
  const generateWorld = (): void => {
    setWorld("generating");

    // Update the config query param
    worldConfigHandler.updateQueryParam();

    // Defer world gen to idle time, so the browser prioritizes UI updates
    window.requestIdleCallback(() => {
      setWorld(generate_world(worldConfigHandler.config));
    });
  };

  return (
    <DemoContext.Provider
      value={{
        worldConfigHandler,
        renderConfigHandler,
        world,
        generateWorld,
      }}
    >
      <Switch>
        <Route path="/demo/new" exact>
          <WorldConfigEditor fullscreen />
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
