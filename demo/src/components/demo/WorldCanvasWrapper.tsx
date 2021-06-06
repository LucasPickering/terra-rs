import React, { useContext, useRef, useState } from "react";
import { IconButton, makeStyles } from "@material-ui/core";
import {
  Edit as IconEdit,
  Close as IconClose,
  ArrowBack as IconArrowBack,
  GetApp as IconGetApp,
} from "@material-ui/icons";
import { Location } from "history";
import DownloadMenu from "./DownloadMenu";
import UnstyledLink from "../UnstyledLink";
import ConfigEditorOverlay from "./ConfigEditorOverlay";
import WorldCanvas from "./WorldCanvas";
import DemoContext from "context/DemoContext";

const useStyles = makeStyles(({ spacing }) => ({
  loading: {
    margin: "auto 0",
  },
  canvasWrapper: {
    position: "relative",
    width: "100%",
    height: "100%",
    overflow: "hidden",
  },
  canvasOverlay: {
    position: "absolute",
    display: "flex",
    flexDirection: "column",
    width: "40%",
    maxWidth: 600,
    maxHeight: "100%",
    padding: spacing(1),
    paddingRight: 0,
  },
  buttons: {
    marginBottom: spacing(1),
  },
}));

const WorldCanvasWrapper: React.FC = () => {
  const classes = useStyles();
  const downloadMenuButtonRef = useRef<HTMLButtonElement | null>(null);
  const [overlayOpen, setOverlayOpen] = useState<boolean>(false);
  const [downloadMenuOpen, setDownloadMenuOpen] = useState<boolean>(false);
  const { worldState } = useContext(DemoContext);

  return (
    <div className={classes.canvasWrapper}>
      <div className={classes.canvasOverlay}>
        <div className={classes.buttons}>
          <IconButton
            aria-label="back to config"
            component={UnstyledLink}
            to={(location: Location) => ({
              ...location,
              pathname: "/demo/new",
            })}
          >
            <IconArrowBack />
          </IconButton>

          <IconButton
            ref={downloadMenuButtonRef}
            aria-controls="download-menu"
            aria-haspopup="true"
            disabled={worldState.phase !== "populated"}
            onClick={() => setDownloadMenuOpen(true)}
          >
            <IconGetApp />
          </IconButton>
          <DownloadMenu
            id="download-menu"
            anchorEl={downloadMenuButtonRef.current}
            open={worldState.phase === "populated" && downloadMenuOpen}
            getContentAnchorEl={null}
            anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
            onClose={() => setDownloadMenuOpen(false)}
          />

          <IconButton
            aria-label={overlayOpen ? "close config" : "edit config"}
            onClick={() => setOverlayOpen((old) => !old)}
          >
            {overlayOpen ? <IconClose /> : <IconEdit />}
          </IconButton>
        </div>
        {overlayOpen && <ConfigEditorOverlay />}
      </div>

      <WorldCanvas />
    </div>
  );
};

export default WorldCanvasWrapper;
