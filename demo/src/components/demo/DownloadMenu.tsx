import React, { useContext } from "react";
import { saveAs } from "file-saver";
import { Menu, MenuItem } from "@material-ui/core";
import { World } from "terra-wasm";
import DemoContext from "context/DemoContext";

const DownloadMenu: React.FC<React.ComponentProps<typeof Menu>> = (props) => {
  const { terra, world, worldConfigHandler, renderConfigHandler } =
    useContext(DemoContext);

  // We _shouldn't_ ever render this if the world isn't present, but just need
  // this check to tell TS that
  if (!(world instanceof World)) {
    return null;
  }

  return (
    <Menu {...props}>
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
          const jsonString = JSON.stringify(worldConfigHandler.config);
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
          const renderer = terra.build_renderer(renderConfigHandler.config);
          const svg = renderer.render_as_svg(world);
          saveAs(new Blob([svg], { type: "image/svg+xml" }), "terra.svg");
        }}
      >
        Download as SVG
      </MenuItem>
      <MenuItem
        onClick={() => {
          const renderer = terra.build_renderer(renderConfigHandler.config);
          const bytes = renderer.render_as_stl(world);
          saveAs(new Blob([bytes], { type: "model/stl" }), "terra.stl");
        }}
      >
        Download as STL
      </MenuItem>
    </Menu>
  );
};

export default DownloadMenu;
