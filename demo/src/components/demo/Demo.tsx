import React, { useState } from "react";
import { Redirect, Route, Switch, useHistory } from "react-router-dom";
import ConfigEditor from "./ConfigEditor";
import WorldCanvas from "./WorldCanvas";
import NotFound from "../NotFound";
import useStaticValue from "hooks/useStaticValue";
import DemoContext, { ConfigKey } from "context/DemoContext";
import type { World, WorldConfigObject } from "terra-wasm";
import useQueryParams from "hooks/useQueryParams";
import { set } from "../../util";
const { Terra } = await import("terra-wasm");

const CONFIG_QUERY_PARAM = "config";

/**
 * Configure and generate a demo Terra world.
 */
const Demo: React.FC = () => {
  const history = useHistory();
  const { params: queryParams } = useQueryParams();

  // Initialize the Terra singleton, which will be our interface into all wasm
  const terra = useStaticValue(() => new Terra());

  // Store the world config as a JS object. We'll deserialize it into a Rust
  // value before world generation.
  const [worldConfig, setWorldConfig] = useState<WorldConfigObject>(() => {
    // If there's a config object in the URL query, use that. If not (or if
    // parsing the query fails), fall back to the default.
    const queryConfigStr = queryParams.get(CONFIG_QUERY_PARAM);
    if (queryConfigStr) {
      try {
        const queryConfigObj = JSON.parse(queryConfigStr);
        // Make sure this is a valid config. If not, this will throw.
        // This will also populate defaults where missing
        return terra.validate_world_config(queryConfigObj);
      } catch (e) {
        // eslint-disable-next-line no-console
        console.warn("Error parsing config from query params:", e);
      }
    }
    return terra.default_world_config();
  });
  const [lastGeneratedConfig, setLastGeneratedConfig] =
    useState<WorldConfigObject | undefined>();

  const [world, setWorld] = useState<World | "generating" | undefined>();
  const generateWorld = async (goToWorld: boolean): Promise<void> => {
    setWorld("generating");
    setLastGeneratedConfig(worldConfig);

    // Update the config query param
    const newParams = new URLSearchParams();
    newParams.set(CONFIG_QUERY_PARAM, JSON.stringify(worldConfig));
    const search = newParams.toString();
    if (goToWorld) {
      history.push({ pathname: "/demo/world", search });
    } else {
      history.replace({ ...history.location, search });
    }

    // Defer world gen to idle time, so the browser prioritizes UI updates
    window.requestIdleCallback(() => {
      setWorld(
        terra.generate_world(terra.deserialize_world_config(worldConfig))
      );
    });
  };

  return (
    <DemoContext.Provider
      value={{
        terra,
        worldConfig,
        renderConfig: terra.default_render_config(),
        generateWorldEnabled:
          world !== "generating" && worldConfig !== lastGeneratedConfig,
        setConfig: setWorldConfig,
        setConfigValue: (key: ConfigKey, value: unknown) => {
          const newConfig = { ...worldConfig }; // Shallow copy to force a rerender
          set(newConfig, key, value);
          setWorldConfig(newConfig);
        },
        resetConfig: () => {
          if (
            window.confirm(
              "Are you sure? You will lose all your current settings."
            )
          ) {
            setWorldConfig(terra.default_world_config());
          }
        },
        world,
        generateWorld,
      }}
    >
      <Switch>
        <Redirect from="/demo" to="/demo/new" exact />

        <Route path="/demo/new" exact>
          <ConfigEditor />
        </Route>

        <Route path="/demo/world" exact>
          <WorldCanvas />
        </Route>

        <Route path="*" exact>
          <NotFound />
        </Route>
      </Switch>
    </DemoContext.Provider>
  );
};

export default Demo;
