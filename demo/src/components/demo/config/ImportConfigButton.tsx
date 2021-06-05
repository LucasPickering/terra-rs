import React from "react";
import { Button } from "@material-ui/core";
import { ConfigHandler } from "hooks/useConfigHandler";

interface Props<T> {
  configHandler: ConfigHandler<T>;
}

function ImportConfigButton<T>({
  configHandler,
}: Props<T>): React.ReactElement {
  /**
   * Load a generic config from the given JSON file
   */
  const loadConfig = async (file: File): Promise<void> => {
    try {
      const json = await file.text();
      configHandler.setFromJson(json);
    } catch (error) {
      window.alert("Failed to load config: " + error);
    }
  };

  return (
    <>
      <Button
        component="label"
        htmlFor="config-file-input"
        fullWidth
        variant="outlined"
      >
        Import Config
      </Button>
      <input
        id="config-file-input"
        type="file"
        accept=".json"
        hidden
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) {
            loadConfig(file);
          }
        }}
      />
    </>
  );
}

export default ImportConfigButton;
