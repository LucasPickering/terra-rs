import React, { useContext, useEffect, useRef } from "react";
import WorldDemo from "3d/WorldDemo";
import { CircularProgress, makeStyles, Typography } from "@material-ui/core";
import DemoContext from "context/DemoContext";
import useDebouncedValue from "hooks/useDebouncedValue";
import Link from "components/Link";

const useStyles = makeStyles(() => ({
  content: {
    width: "100%",
    height: "100%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
}));

/**
 * Render the given Terra world in 3D. If the world is undefined, we assume it's
 * still loading. This is the last line of defense in the Kingdom of React;
 * everything below this belongs to the filthy peasants of Babylon.js-topia.
 */
const WorldCanvas: React.FC = () => {
  const { renderConfigHandler, worldState, generateWorld } =
    useContext(DemoContext);
  const classes = useStyles();
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const worldDemoRef = useRef<WorldDemo | undefined>();

  // If we ever hit this page with no world present, then generate one
  const worldPhase = worldState.phase;
  useEffect(() => {
    if (worldPhase === "empty") {
      generateWorld();
    }
    // generateWorld is unstable, this is a hack to get around that
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [worldPhase]);

  // Update render state whenever the world state changes
  useEffect(() => {
    // If we have a demo rendered but the world is gone, dump the render
    if (worldState.phase !== "populated" && worldDemoRef.current) {
      worldDemoRef.current.dispose();
      worldDemoRef.current = undefined;
    }

    if (canvasRef.current && worldState.phase === "populated") {
      // The above check should always dispose of the last render, but this is
      // just a safety check in case that branch never got called
      if (worldDemoRef.current) {
        worldDemoRef.current.dispose();
      }

      // World is ready, render it.
      worldDemoRef.current = new WorldDemo(
        canvasRef.current,
        worldState.world,
        renderConfigHandler.config
      );
    }
  }, [renderConfigHandler, worldState]);

  // Whenever the render config changes, re-render the world. The config should
  // only change when a user actually provides input, not on every React
  // re-render. And debounce the changes so dragging a slider doesn't trigger
  // a ton of updates.
  const debouncedRenderConfig = useDebouncedValue(
    renderConfigHandler.config,
    500
  );
  const { updateQueryParam } = renderConfigHandler;
  useEffect(() => {
    worldDemoRef.current?.updateRenderConfig(debouncedRenderConfig);
    updateQueryParam();
  }, [debouncedRenderConfig, worldDemoRef, updateQueryParam]);

  if (worldState.phase === "generating") {
    return (
      <div className={classes.content}>
        <CircularProgress size="10rem" />
      </div>
    );
  }

  if (worldState.phase === "error") {
    // Big sad
    return (
      <div className={classes.content}>
        <div>
          <Typography variant="h3">
            An error occurred during world generation :(
          </Typography>
          <Typography>
            Please{" "}
            <Link to="https://github.com/LucasPickering/terra-rs/issues/new">
              file an issue
            </Link>{" "}
            for this and <strong>include the following:</strong>
            <ul>
              <li>
                The world generation config JSON (available in the config editor
                panel on this page)
              </li>
              <li>The error from the browser developer console</li>
            </ul>
          </Typography>
        </div>
      </div>
    );
  }

  return <canvas ref={canvasRef} />;
};

export default WorldCanvas;
