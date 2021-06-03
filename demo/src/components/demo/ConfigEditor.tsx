import React, { useContext } from "react";
import {
  Button,
  Grid,
  GridSize,
  MenuItem,
  Typography,
} from "@material-ui/core";
import ConfigInput from "./ConfigInput";
import DemoContext from "context/DemoContext";
import ConfigSection from "./ConfigSection";
import RangeConfigInput from "./RangeConfigInput";
import SelectConfigInput from "./SelectConfigInput";
import TextConfigInput from "./TextConfigInput";
import ImportConfigButton from "./ImportConfigButton";
import descriptions from "./descriptions";
import { Breakpoint } from "@material-ui/core/styles/createBreakpoints";

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

function formatMeter3(value: number): string {
  return `${value} m³`;
}

/**
 * Edit the current world config
 */
const ConfigEditor: React.FC<{ inline?: boolean }> = ({ inline = false }) => {
  const { generateWorldEnabled, resetConfig, generateWorld } =
    useContext(DemoContext);
  // TODO figure out how to make this properly dynamic so breakpoints are based
  // on parent size rather than screen size
  const sectionSize: Partial<Record<Breakpoint, GridSize>> = inline
    ? { xs: 12 }
    : { xs: 12, md: 6 };
  const buttonSize: Partial<Record<Breakpoint, GridSize>> = inline
    ? { xs: 12 }
    : { xs: 12, sm: 6, md: 4, lg: 3, xl: 2 };

  // Min/max values are based from config.rs, with some extra restrictions where
  // necessary (i.e. some values don't always have a min/max so we define our
  // own in order to use type="range").
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault(); // Don't reload the page
        generateWorld(true);
      }}
    >
      <Grid container spacing={4}>
        <Grid item xs={12}>
          <Typography variant="h2">Configure World</Typography>
        </Grid>

        <Grid item xs={12} container spacing={1} justify="flex-end">
          <Grid item {...buttonSize}>
            <Button fullWidth variant="outlined" onClick={resetConfig}>
              Reset to Default
            </Button>
          </Grid>
          <Grid item {...buttonSize}>
            <ImportConfigButton />
          </Grid>
          <Grid item {...buttonSize}>
            <Button
              disabled={!generateWorldEnabled}
              fullWidth
              type="submit"
              color="primary"
              variant="contained"
            >
              {inline ? "Regenerate World" : "Generate World"}
            </Button>
          </Grid>
        </Grid>

        <Grid item {...sectionSize}>
          <ConfigSection title="General" description="General world settings">
            <ConfigInput
              field={["seed"]}
              label="Random Seed"
              description={descriptions.general.seed}
            >
              <TextConfigInput />
            </ConfigInput>
            <ConfigInput
              field={["radius"]}
              label="World Radius"
              description={descriptions.general.radius}
            >
              <RangeConfigInput min={0} max={500} step={10} />
            </ConfigInput>
          </ConfigSection>
        </Grid>

        <Grid item {...sectionSize}>
          <ConfigSection
            title="Edge Buffer"
            description={descriptions.edge_buffer.root}
          >
            <ConfigInput
              field={["edge_buffer_fraction"]}
              label="Edge Buffer Fraction"
              description={descriptions.edge_buffer.edge_buffer_fraction}
            >
              <RangeConfigInput {...NORMAL_RANGE} />
            </ConfigInput>
            <ConfigInput
              field={["edge_buffer_exponent"]}
              label="Edge Buffer Exponent"
              description={descriptions.edge_buffer.edge_buffer_exponent}
            >
              <RangeConfigInput {...EXPONENT_RANGE} />
            </ConfigInput>
          </ConfigSection>
        </Grid>

        <Grid item {...sectionSize}>
          <ConfigSection
            title="Elevation"
            description={descriptions.elevation.root}
          >
            <ConfigInput
              field={["elevation", "noise_type"]}
              label="Noise Type"
              description={descriptions.elevation.noise_type}
            >
              <SelectConfigInput>
                <MenuItem value="basic_multi">Basic Multi</MenuItem>
                <MenuItem value="billow">Billow</MenuItem>
                <MenuItem value="fbm">Fractal Brownian Motion</MenuItem>
                <MenuItem value="hybrid_multi">Hybrid Multi</MenuItem>
                <MenuItem value="ridged_multi">Ridged Multi</MenuItem>
              </SelectConfigInput>
            </ConfigInput>
            <ConfigInput
              field={["elevation", "octaves"]}
              label="Octaves"
              description={descriptions.elevation.octaves}
            >
              <RangeConfigInput min={1} max={10} />
            </ConfigInput>
            <ConfigInput
              field={["elevation", "frequency"]}
              label="Frequency"
              description={descriptions.elevation.frequency}
            >
              <RangeConfigInput min={0.1} max={5.0} step={0.1} />
            </ConfigInput>
            <ConfigInput
              field={["elevation", "lacunarity"]}
              label="Lacunarity"
              description={descriptions.elevation.lacunarity}
            >
              <RangeConfigInput min={0.5} max={10.0} step={0.5} />
            </ConfigInput>
            <ConfigInput
              field={["elevation", "persistence"]}
              label="Persistence"
              description={descriptions.elevation.persistence}
            >
              <RangeConfigInput min={0.0} max={2.0} step={0.1} />
            </ConfigInput>
            <ConfigInput
              field={["elevation", "exponent"]}
              label="Exponent"
              description={descriptions.elevation.exponent}
            >
              <RangeConfigInput {...EXPONENT_RANGE} />
            </ConfigInput>
          </ConfigSection>
        </Grid>

        <Grid item {...sectionSize}>
          <ConfigSection
            title="Rainfall"
            description={descriptions.rainfall.root}
          >
            <ConfigInput
              field={["rainfall", "evaporation_default"]}
              label="Default Evaporation Volume"
              description={descriptions.rainfall.evaporation_default}
            >
              <RangeConfigInput
                min={0.0}
                max={10.0}
                step={0.5}
                formatMark={formatMeter3}
              />
            </ConfigInput>
            <ConfigInput
              field={["rainfall", "evaporation_land_scale"]}
              label="Land Evaporation Scale"
              description={descriptions.rainfall.evaporation_land_scale}
            >
              <RangeConfigInput {...NORMAL_RANGE} />
            </ConfigInput>
            <ConfigInput
              field={["rainfall", "evaporation_spread_distance"]}
              label="Evaporation Spread Distance"
              description={descriptions.rainfall.evaporation_spread_distance}
            >
              <RangeConfigInput min={0} max={100} step={5} />
            </ConfigInput>
            <ConfigInput
              field={["rainfall", "evaporation_spread_exponent"]}
              label="Evaporation Spread Exponent"
              description={descriptions.rainfall.evaporation_spread_exponent}
            >
              <RangeConfigInput {...EXPONENT_RANGE} />
            </ConfigInput>
            <ConfigInput
              field={["rainfall", "rainfall_fraction_limit"]}
              label="Rainfall Fraction Limit"
              description={descriptions.rainfall.rainfall_fraction_limit}
            >
              <RangeConfigInput min={0.0} max={0.5} step={0.05} />
            </ConfigInput>
          </ConfigSection>
        </Grid>

        <Grid item {...sectionSize}>
          <ConfigSection
            title="Geographic Features"
            description={descriptions.geo_feature.root}
          >
            <ConfigInput
              field={["geo_feature", "lake_runoff_threshold"]}
              label="Lake Runoff Threshold"
              description={descriptions.geo_feature.lake_runoff_threshold}
            >
              <RangeConfigInput
                min={0.0}
                max={100.0}
                step={5.0}
                formatMark={formatMeter3}
              />
            </ConfigInput>
            <ConfigInput
              field={["geo_feature", "river_runoff_traversed_threshold"]}
              label="River Runoff-Traversed Threshold"
              description={
                descriptions.geo_feature.river_runoff_traversed_threshold
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
        </Grid>

        <Grid item {...sectionSize}>
          <ConfigSection
            title="Render Options"
            description={descriptions.render.root}
          >
            <ConfigInput
              field={["render", "y_scale"]}
              label="Vertical Scale"
              description={descriptions.render.y_scale}
            >
              <RangeConfigInput
                min={0.1}
                max={10.0}
                step={null}
                markValues={[0.1, 0.5, 1.0, 2.0, 5.0, 10.0]}
              />
            </ConfigInput>
          </ConfigSection>
        </Grid>
      </Grid>
    </form>
  );
};

export default ConfigEditor;
