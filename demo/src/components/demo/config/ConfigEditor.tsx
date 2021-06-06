import React, { useState } from "react";
import { Button, Grid, Typography } from "@material-ui/core";
import { ConfigHandler } from "hooks/useConfigHandler";
import ConfigJsonEditor from "./ConfigJsonEditor";

interface Props<T> {
  configHandler: ConfigHandler<T>;
  title: string;
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
  onSubmit,
  submitButton,
  children,
}: Props<T>): React.ReactElement {
  const [editAsJson, setEditAsJson] = useState<boolean>(false);

  return (
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
          <Grid item xs={6}>
            <Button
              fullWidth
              variant="outlined"
              onClick={() => configHandler.reset(true)}
            >
              Reset to Default
            </Button>
          </Grid>
          <Grid item xs={6}>
            <Button
              fullWidth
              variant="outlined"
              onClick={() => setEditAsJson((old) => !old)}
            >
              Edit as JSON
            </Button>
          </Grid>
          {submitButton && (
            <Grid item xs={12}>
              {submitButton}
            </Grid>
          )}
        </Grid>
        {editAsJson ? (
          <Grid item xs={12} container spacing={1}>
            <ConfigJsonEditor configHandler={configHandler} />
          </Grid>
        ) : (
          children
        )}
      </Grid>
    </form>
  );
}

export default ConfigEditor;
