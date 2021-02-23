import React, { useContext, useEffect, useRef, useState } from "react";
import WorldDemo from "3d/WorldDemo";
import {
  IconButton,
  CircularProgress,
  makeStyles,
  Paper,
} from "@material-ui/core";
import {
  Edit as IconEdit,
  Close as IconClose,
  ArrowBack as IconArrowBack,
} from "@material-ui/icons";
import DemoContext from "context/DemoContext";
import { Location } from "history";
import ConfigEditor from "./ConfigEditor";
import UnstyledLink from "../UnstyledLink";

const useStyles = makeStyles(({ spacing }) => ({
  loading: {
    margin: "auto 0",
  },
  canvasWrapper: {
    position: "relative",
    width: "100%",
    height: "100%",
    overflow: "hidden",
  },
  canvasOverlay: {
    position: "absolute",
    display: "flex",
    flexDirection: "column",
    width: "40%",
    maxWidth: 600,
    maxHeight: "100%",
    padding: spacing(1),
    paddingRight: 0,
  },
  buttons: {
    marginBottom: spacing(1),
  },
  configOverlay: {
    overflowY: "auto",
    padding: `${spacing(1)}px ${spacing(4)}px`,
  },
}));

/**
 * Render the given Terra world in 3D. If the world is undefined, we assume it's
 * still loading. This is the last line of defense in the Kingdom of React;
 * everything below this belongs to the filthy peasants of Babylon.js-topia.
 */
const WorldCanvas: React.FC = () => {
  const { terra, world, generateWorld } = useContext(DemoContext);
  const classes = useStyles();
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const worldDemoRef = useRef<WorldDemo | undefined>();
  const [configOpen, setConfigOpen] = useState<boolean>(false);

  // If we ever hit this page with no world present, then generate one
  useEffect(() => {
    if (world === undefined) {
      generateWorld(false);
    }
    // generateWorld is unstable, this is a hack to get around that
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [world]);

  useEffect(() => {
    // If we have a demo rendered but the world is gone, dump the render
    if (
      (world === undefined || world === "generating") &&
      worldDemoRef.current
    ) {
      worldDemoRef.current.dispose();
      worldDemoRef.current = undefined;
    }

    if (canvasRef.current && world && world !== "generating") {
      // The above check should always dispose of the last render, but this is
      // just a safety check in case that branch never got called
      if (worldDemoRef.current) {
        worldDemoRef.current.dispose();
      }

      // World is ready, render it.
      worldDemoRef.current = new WorldDemo(canvasRef.current, terra, world);
    }
  }, [terra, world]);

  if (world === "generating") {
    return <CircularProgress className={classes.loading} size="10rem" />;
  }

  return (
    <div className={classes.canvasWrapper}>
      <div className={classes.canvasOverlay}>
        <div className={classes.buttons}>
          <IconButton
            aria-label="back to config"
            component={UnstyledLink}
            to={(location: Location) => ({
              ...location,
              pathname: "/demo/new",
            })}
          >
            <IconArrowBack />
          </IconButton>
          <IconButton
            aria-label={configOpen ? "close config" : "edit config"}
            onClick={() => setConfigOpen((old) => !old)}
          >
            {configOpen ? <IconClose /> : <IconEdit />}
          </IconButton>
        </div>
        {configOpen && (
          <Paper className={classes.configOverlay}>
            <ConfigEditor inline />
          </Paper>
        )}
      </div>

      <canvas ref={canvasRef} />
    </div>
  );
};

export default WorldCanvas;
