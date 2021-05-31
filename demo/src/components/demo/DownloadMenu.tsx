import React, { useContext } from "react";
import { saveAs } from "file-saver";
import { Menu, MenuItem } from "@material-ui/core";
import type { WasmWorld } from "terra-wasm";
import DemoContext from "context/DemoContext";
const { TileLens } = await import("terra-wasm");

interface Props extends React.ComponentProps<typeof Menu> {
  world: WasmWorld;
}

const DownloadMenu: React.FC<Props> = ({ world, ...rest }) => {
  const { config } = useContext(DemoContext);

  return (
    <Menu {...rest}>
      <MenuItem
        onClick={() => {
          const jsonString = world.to_json();
          saveAs(
            new Blob([jsonString], { type: "application/json" }),
            "terra.json"
          );
        }}
      >
        Download as JSON
      </MenuItem>
      <MenuItem
        onClick={() => {
          const jsonString = JSON.stringify(config);
          saveAs(
            new Blob([jsonString], { type: "application/json" }),
            "terra_config.json"
          );
        }}
      >
        Download as JSON (Config Only)
      </MenuItem>
      <MenuItem
        onClick={() => {
          const bytes = world.to_bin();
          saveAs(
            new Blob([bytes], { type: "application/octet-stream" }),
            "terra.bin"
          );
        }}
      >
        Download as BIN
      </MenuItem>
      <MenuItem
        onClick={() => {
          const svg = world.to_svg(TileLens.Biome, true);
          saveAs(new Blob([svg], { type: "image/svg+xml" }), "terra.svg");
        }}
      >
        Download as SVG
      </MenuItem>
      <MenuItem
        onClick={() => {
          const bytes = world.to_stl();
          saveAs(new Blob([bytes], { type: "model/stl" }), "terra.stl");
        }}
      >
        Download as STL
      </MenuItem>
    </Menu>
  );
};

export default DownloadMenu;
