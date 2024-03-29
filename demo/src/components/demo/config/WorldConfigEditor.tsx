import React, { useContext } from "react";
import { Button, MenuItem } from "@material-ui/core";
import ConfigInput from "./ConfigInput";
import DemoContext from "context/DemoContext";
import ConfigSection from "./ConfigSection";
import RangeConfigInput from "./RangeConfigInput";
import SelectConfigInput from "./SelectConfigInput";
import TextConfigInput from "./TextConfigInput";
import { worldDescriptions } from "./descriptions";
import ConfigEditor from "./ConfigEditor";
import type { WorldConfigObject } from "terra";
import { formatMeter3 } from "../../../util";
import { Redirect, useLocation } from "react-router";
import CheckboxConfigInput from "./CheckboxConfigInput";

const NORMAL_RANGE = {
  min: 0.0,
  max: 1.0,
  step: 0.05,
};
const EXPONENT_RANGE = {
  min: 0.0,
  max: 3.0,
  step: 0.1,
};

/**
 * Edit the current world config
 */
const WorldConfigEditor: React.FC<{ standalone?: boolean }> = ({
  standalone = false,
}) => {
  const location = useLocation();
  const { worldConfigHandler, worldState, generateWorld } =
    useContext(DemoContext);

  // If we have started (or finished) generating a world already, then go to
  // the world screen. We only want to be on the config screen if no world
  // exists yet
  if (standalone && worldState.phase !== "empty") {
    // TODO https://github.com/LucasPickering/terra-rs/issues/30
    // Currently this triggers a re-mount of Demo which leads to the world
    // getting thrown away and generated a 2nd time. It really shouldn't be
    // re-mounting and I can't figure out why it is.
    return <Redirect to={{ ...location, pathname: "/demo/world" }} />;
  }

  // Min/max values are based from config.rs, with some extra restrictions where
  // necessary (i.e. some values don't always have a min/max so we define our
  // own in order to use type="range").
  return (
    <ConfigEditor<WorldConfigObject>
      configHandler={worldConfigHandler}
      title="Configure World Generation"
      onSubmit={() => generateWorld()}
      submitButton={
        <Button
          disabled={worldState.phase === "generating"}
          fullWidth
          type="submit"
          color="primary"
          variant="contained"
        >
          {standalone ? "Generate World" : "Regenerate World"}
        </Button>
      }
    >
      <ConfigSection title="General" description="General world settings">
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["seed"]}
          label="Random Seed"
          description={worldDescriptions.general.seed}
        >
          <TextConfigInput />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["radius"]}
          label="World Radius"
          description={worldDescriptions.general.radius}
        >
          <RangeConfigInput min={0} max={500} step={10} />
        </ConfigInput>
      </ConfigSection>

      <ConfigSection
        title="Edge Buffer"
        description={worldDescriptions.edge_buffer.root}
      >
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "edge_buffer_fraction"]}
          label="Edge Buffer Fraction"
          description={worldDescriptions.edge_buffer.edge_buffer_fraction}
        >
          <RangeConfigInput {...NORMAL_RANGE} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "edge_buffer_exponent"]}
          label="Edge Buffer Exponent"
          description={worldDescriptions.edge_buffer.edge_buffer_exponent}
        >
          <RangeConfigInput {...EXPONENT_RANGE} />
        </ConfigInput>
      </ConfigSection>

      <ConfigSection
        title="Elevation"
        description={worldDescriptions.elevation.root}
      >
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "noise_fn", "noise_type"]}
          label="Noise Type"
          description={worldDescriptions.elevation.noise_type}
        >
          <SelectConfigInput>
            <MenuItem value="basic_multi">Basic Multi</MenuItem>
            <MenuItem value="billow">Billow</MenuItem>
            <MenuItem value="fbm">Fractal Brownian Motion</MenuItem>
            <MenuItem value="hybrid_multi">Hybrid Multi</MenuItem>
            <MenuItem value="ridged_multi">Ridged Multi</MenuItem>
          </SelectConfigInput>
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "noise_fn", "octaves"]}
          label="Octaves"
          description={worldDescriptions.elevation.octaves}
        >
          <RangeConfigInput min={1} max={10} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "noise_fn", "frequency"]}
          label="Frequency"
          description={worldDescriptions.elevation.frequency}
        >
          <RangeConfigInput min={0.1} max={5.0} step={0.1} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "noise_fn", "lacunarity"]}
          label="Lacunarity"
          description={worldDescriptions.elevation.lacunarity}
        >
          <RangeConfigInput min={0.5} max={10.0} step={0.5} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "noise_fn", "persistence"]}
          label="Persistence"
          description={worldDescriptions.elevation.persistence}
        >
          <RangeConfigInput min={0.0} max={2.0} step={0.1} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["elevation", "noise_fn", "exponent"]}
          label="Exponent"
          description={worldDescriptions.elevation.exponent}
        >
          <RangeConfigInput {...EXPONENT_RANGE} />
        </ConfigInput>
        {/* Excluding rounding_interval for now because it breaks runoff gen */}
      </ConfigSection>

      <ConfigSection
        title="Rainfall"
        description={worldDescriptions.rainfall.root}
      >
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["rainfall", "enabled"]}
          label="Enabled?"
          description={worldDescriptions.rainfall.enabled}
        >
          <CheckboxConfigInput />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["rainfall", "evaporation_default"]}
          label="Default Evaporation Volume"
          description={worldDescriptions.rainfall.evaporation_default}
        >
          <RangeConfigInput
            min={0.0}
            max={10.0}
            step={0.5}
            formatMark={formatMeter3}
          />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["rainfall", "evaporation_land_scale"]}
          label="Land Evaporation Scale"
          description={worldDescriptions.rainfall.evaporation_land_scale}
        >
          <RangeConfigInput {...NORMAL_RANGE} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["rainfall", "evaporation_spread_distance"]}
          label="Evaporation Spread Distance"
          description={worldDescriptions.rainfall.evaporation_spread_distance}
        >
          <RangeConfigInput min={0} max={100} step={5} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["rainfall", "evaporation_spread_exponent"]}
          label="Evaporation Spread Exponent"
          description={worldDescriptions.rainfall.evaporation_spread_exponent}
        >
          <RangeConfigInput {...EXPONENT_RANGE} />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["rainfall", "rainfall_fraction_limit"]}
          label="Rainfall Fraction Limit"
          description={worldDescriptions.rainfall.rainfall_fraction_limit}
        >
          <RangeConfigInput min={0.0} max={0.5} step={0.05} />
        </ConfigInput>
      </ConfigSection>

      <ConfigSection
        title="Geographic Features"
        description={worldDescriptions.geo_feature.root}
      >
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["geo_feature", "lake_runoff_threshold"]}
          label="Lake Runoff Threshold"
          description={worldDescriptions.geo_feature.lake_runoff_threshold}
        >
          <RangeConfigInput
            min={0.0}
            max={20.0}
            step={1.0}
            formatMark={formatMeter3}
          />
        </ConfigInput>
        <ConfigInput<WorldConfigObject>
          configHandler={worldConfigHandler}
          field={["geo_feature", "river_runoff_traversed_threshold"]}
          label="River Runoff-Traversed Threshold"
          description={
            worldDescriptions.geo_feature.river_runoff_traversed_threshold
          }
        >
          <RangeConfigInput
            min={0.0}
            max={1000.0}
            step={50.0}
            formatMark={formatMeter3}
          />
        </ConfigInput>
      </ConfigSection>
    </ConfigEditor>
  );
};

export default WorldConfigEditor;
