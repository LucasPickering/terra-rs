import { makeStyles, Typography } from "@material-ui/core";
import React from "react";
import { useRouteMatch } from "react-router-dom";
import Link from "./Link";

/**
 * These routes won't get the footer or the width restriction
 */
const FULLSCREEN_ROUTES = ["/demo/world"];

const useStyles = makeStyles(({ breakpoints, palette, spacing }) => ({
  // The entire page
  pageLayout: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    height: "100%",
  },

  // The width-restricted column that holds the body+footer
  pageColumn: {
    flexGrow: 1, // Fill all vertical space
    // Force the footer to the bottom
    display: "flex",
    flexDirection: "column",
    justifyContent: "space-between",

    width: "100%",
    [breakpoints.up("xs")]: {
      maxWidth: "100%", // 12/12
    },
    [breakpoints.up("md")]: {
      maxWidth: "83%", // 10/12
    },
    [breakpoints.up("lg")]: {
      maxWidth: "67%", // 8/12
    },

    padding: `${spacing(2)}px ${spacing(4)}px`,
    backgroundColor: palette.background.default,
  },

  header: {
    display: "flex",
    alignItems: "end",
    margin: `${spacing(2)}px 0`,
    "& > *": {
      margin: 0,
    },
    "& > :not(:first-child)": {
      marginLeft: spacing(2),
    },
  },

  footer: {
    display: "flex",
    justifyContent: "center",
    marginTop: spacing(2),
    "& > :not(:first-child)": {
      marginLeft: spacing(2),
    },
  },
}));

/**
 * Container for all content on the page. This is used in the root to wrap all
 * pages.
 */
const PageLayout: React.FC = ({ children }) => {
  const classes = useStyles();
  const fullscreenMatch = useRouteMatch(FULLSCREEN_ROUTES);

  // Fullscreen routes don't need any fancy CSS
  if (fullscreenMatch) {
    return <div className={classes.pageLayout}>{children}</div>;
  }

  return (
    <div className={classes.pageLayout}>
      <div className={classes.pageColumn}>
        {/* Header */}
        <div>
          <header className={classes.header}>
            <Typography variant="h1">Terra</Typography>
            <Typography variant="subtitle1">
              Terrain Generation System
            </Typography>
          </header>

          {children}
        </div>

        <footer className={classes.footer}>
          <Typography variant="body2">
            Created by{" "}
            <Link to="https://lucaspickering.me">Lucas Pickering</Link>
          </Typography>
          <Link to="https://github.com/LucasPickering/terra-rs">GitHub</Link>
        </footer>
      </div>
    </div>
  );
};

export default PageLayout;
