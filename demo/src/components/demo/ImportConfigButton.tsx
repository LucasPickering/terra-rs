import React, { useContext } from "react";
import { Button } from "@material-ui/core";
import DemoContext from "context/DemoContext";

const ImportConfigButton: React.FC = () => {
  const { terra, setConfig } = useContext(DemoContext);

  /**
   * Load a terra config from the given file
   */
  const loadConfig = async (file: File): Promise<void> => {
    try {
      const data = await file.text();
      const configString = JSON.parse(data);
      const validConfig = terra.validate_world_config(configString);
      setConfig(validConfig);
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
};

export default ImportConfigButton;
