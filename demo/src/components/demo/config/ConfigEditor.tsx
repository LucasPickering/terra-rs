import React from "react";
import { Button, Grid, GridSize, Typography } from "@material-ui/core";
import ImportConfigButton from "./ImportConfigButton";
import { Breakpoint } from "@material-ui/core/styles/createBreakpoints";
import { ConfigHandler } from "hooks/useConfigHandler";
import ConfigEditorContext from "./ConfigEditorContext";

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
            <Typography variant="h2">{title}</Typography>
          </Grid>
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
            <Grid item {...buttonSize}>
              <ImportConfigButton configHandler={configHandler} />
            </Grid>
            {submitButton && (
              <Grid item {...buttonSize}>
                {submitButton}
              </Grid>
            )}
          </Grid>
          {children}
        </Grid>
      </form>
    </ConfigEditorContext.Provider>
  );
}

export default ConfigEditor;
