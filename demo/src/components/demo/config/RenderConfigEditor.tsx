import React, { useContext } from "react";
import { MenuItem } from "@material-ui/core";
import ConfigInput from "./ConfigInput";
import DemoContext from "context/DemoContext";
import ConfigSection from "./ConfigSection";
import RangeConfigInput from "./RangeConfigInput";
import SelectConfigInput from "./SelectConfigInput";
import ConfigEditor from "./ConfigEditor";
import { RenderConfigObject } from "terra-wasm";
import { renderDescriptions } from "./descriptions";

/**
 * Edit the current render config
 */
const RenderConfigEditor: React.FC = () => {
  const { renderConfigHandler } = useContext(DemoContext);

  // Min/max values are based from config.rs, with some extra restrictions where
  // necessary (i.e. some values don't always have a min/max so we define our
  // own in order to use type="range").
  return (
    <ConfigEditor<RenderConfigObject>
      configHandler={renderConfigHandler}
      title="Configure Render Settings"
      fullscreen={false}
    >
      <ConfigSection
        title="Render Settings"
        description={renderDescriptions.root}
      >
        <ConfigInput<RenderConfigObject>
          configHandler={renderConfigHandler}
          field={["vertical_scale"]}
          label="Vertical Scale"
          description={renderDescriptions.vertical_scale}
        >
          <RangeConfigInput min={0.1} max={5.0} step={0.1} />
        </ConfigInput>

        <ConfigInput<RenderConfigObject>
          configHandler={renderConfigHandler}
          field={["tile_lens"]}
          label="Tile Lens"
          description={renderDescriptions.tile_lens}
        >
          <SelectConfigInput>
            <MenuItem value="surface">Surface</MenuItem>
            <MenuItem value="biome">Biome</MenuItem>
            <MenuItem value="elevation">Elevation</MenuItem>
            <MenuItem value="humidity">Humidity</MenuItem>
            <MenuItem value="runoff">Runoff</MenuItem>
          </SelectConfigInput>
        </ConfigInput>

        {/* Skipping show_features since we don't show any here anyway */}
      </ConfigSection>
    </ConfigEditor>
  );
};

export default RenderConfigEditor;
