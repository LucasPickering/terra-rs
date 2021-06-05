import React, { useState } from "react";
import {
  Button,
  Grid,
  GridSize,
  IconButton,
  makeStyles,
  Typography,
} from "@material-ui/core";
import { Breakpoint } from "@material-ui/core/styles/createBreakpoints";
import { ConfigHandler } from "hooks/useConfigHandler";
import { Description as IconDescription } from "@material-ui/icons";
import ConfigEditorContext from "./ConfigEditorContext";
import ConfigJsonEditor from "./ConfigJsonEditor";

const useStyles = makeStyles(() => ({
  header: {
    display: "flex",
    justifyContent: "space-between",
  },
}));

interface Props<T> {
  configHandler: ConfigHandler<T>;
  title: string;
  fullscreen: boolean;
  onSubmit?: () => void;
  submitButton?: React.ReactElement;
  children?: React.ReactNode;
}

/**
 * Generic component for editing a config. The actual config controls should
 * be passed as a child.
 */
function ConfigEditor<T>({
  configHandler,
  title,
  fullscreen,
  onSubmit,
  submitButton,
  children,
}: Props<T>): React.ReactElement {
  const classes = useStyles();
  const [editAsJson, setEditAsJson] = useState<boolean>(false);
  const buttonSize: Partial<Record<Breakpoint, GridSize>> = fullscreen
    ? { xs: 12, sm: 6, md: 4, lg: 3, xl: 2 }
    : { xs: 12 };

  return (
    <ConfigEditorContext.Provider value={{ fullscreen }}>
      <form
        onSubmit={(e) => {
          e.preventDefault(); // Don't reload the page
          if (onSubmit) {
            onSubmit();
          }
        }}
      >
        <Grid container spacing={4}>
          <Grid item xs={12}>
            <div className={classes.header}>
              <Typography variant="h2">{title}</Typography>
              <IconButton onClick={() => setEditAsJson((old) => !old)}>
                <IconDescription />
              </IconButton>
            </div>
          </Grid>
          {editAsJson ? (
            <Grid item xs={12} container spacing={1}>
              <ConfigJsonEditor configHandler={configHandler} />
            </Grid>
          ) : (
            <>
              <Grid item xs={12} container spacing={1} justify="flex-end">
                <Grid item {...buttonSize}>
                  <Button
                    fullWidth
                    variant="outlined"
                    onClick={() => configHandler.reset(true)}
                  >
                    Reset to Default
                  </Button>
                </Grid>
                {submitButton && (
                  <Grid item {...buttonSize}>
                    {submitButton}
                  </Grid>
                )}
              </Grid>
              {children}
            </>
          )}
        </Grid>
      </form>
    </ConfigEditorContext.Provider>
  );
}

export default ConfigEditor;
