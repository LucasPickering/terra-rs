import React, { useEffect, useMemo, useState } from "react";
import { Button, makeStyles, TextField } from "@material-ui/core";
import { ConfigHandler } from "hooks/useConfigHandler";

const useStyles = makeStyles(() => ({
  saveButton: {
    width: "100%",
  },
  textField: {
    width: "100%",
    height: "100%",
  },
}));

interface Props<T> {
  configHandler: ConfigHandler<T>;
}

/**
 * Generic component for editing a config. The actual config controls should
 * be passed as a child.
 */
function ConfigJsonEditor<T>({ configHandler }: Props<T>): React.ReactElement {
  const classes = useStyles();
  const [currentJson, setCurrentJson] = useState<string>(() =>
    toJson(configHandler.config)
  );
  const [error, setError] = useState<Error | undefined>(undefined);

  // Whenever the outer config changes, refresh our version here. This will
  // update what we have internally after a save
  useEffect(() => {
    setCurrentJson(toJson(configHandler.config));
  }, [configHandler.config]);

  // Whenever the user makes an edit, clear the current error (if any). If the
  // config is still invalid, the error will be re-populated when they next
  // try to save.
  useEffect(() => {
    setError(undefined);
  }, [currentJson]);

  // Parse the current JSON value and update the outer state
  const save = (): void => {
    // If the config fails to parse, we'll show an error to the user
    try {
      configHandler.setFromJson(currentJson);
    } catch (error) {
      setError(error);
    }
  };

  const hasError = Boolean(error);
  // Track whether the user has made changes to the config. If not, then
  // we'll disable the save button
  const hasChanges = useMemo(
    () => currentJson === toJson(configHandler.config),
    [currentJson, configHandler.config]
  );

  return (
    <>
      <Button
        className={classes.saveButton}
        disabled={hasChanges || hasError}
        color="primary"
        variant="contained"
        onClick={() => save()}
      >
        Save
      </Button>
      <TextField
        className={classes.textField}
        error={hasError}
        variant="outlined"
        helperText={error?.toString()}
        multiline
        value={currentJson}
        onChange={(e) => setCurrentJson(e.target.value)}
      />
    </>
  );
}

function toJson(config: unknown): string {
  return JSON.stringify(config, null, 4);
}

export default ConfigJsonEditor;
