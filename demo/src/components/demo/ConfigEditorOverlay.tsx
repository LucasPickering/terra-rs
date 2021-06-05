import React, { useState } from "react";
import { makeStyles, Paper, Tabs, Tab, AppBar } from "@material-ui/core";
import RenderConfigEditor from "./config/RenderConfigEditor";
import WorldConfigEditor from "./config/WorldConfigEditor";

const useStyles = makeStyles(({ spacing }) => ({
  configOverlay: {
    overflowY: "auto",
  },
  configOverlayPaper: {
    padding: `${spacing(1)}px ${spacing(4)}px 0 ${spacing(4)}px`,
  },
}));

/**
 * An overlay rendered on top of the world canvas that allows the user to
 * edit the world and render configs.
 */
const ConfigEditorOverlay: React.FC = () => {
  const classes = useStyles();
  const [openTab, setOpenTab] =
    useState<"worldConfig" | "renderConfig">("worldConfig");

  return (
    <div className={classes.configOverlay}>
      <AppBar position="sticky">
        <Tabs value={openTab} onChange={(e, newTab) => setOpenTab(newTab)}>
          <Tab label="World" value="worldConfig" />
          <Tab label="Render" value="renderConfig" />
        </Tabs>
      </AppBar>
      <Paper className={classes.configOverlayPaper} square>
        {openTab === "worldConfig" && <WorldConfigEditor />}
        {openTab === "renderConfig" && <RenderConfigEditor />}
      </Paper>
    </div>
  );
};

export default ConfigEditorOverlay;
