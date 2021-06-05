import React, { useContext } from "react";
import { Grid, GridSize, makeStyles, Typography } from "@material-ui/core";
import HelpTooltip from "components/HelpTooltip";
import ConfigEditorContext from "./ConfigEditorContext";
import { Breakpoint } from "@material-ui/core/styles/createBreakpoints";

const useStyles = makeStyles(({ spacing }) => ({
  configSection: {
    display: "flex",
    flexDirection: "column",
    width: "100%",
  },
  titleWrapper: {
    display: "flex",
    alignItems: "center",
    width: "100%",
  },
  titleText: {
    marginRight: spacing(1),
  },
}));

const ConfigSection = ({
  title,
  description,
  children,
  ...rest
}: {
  title: string;
  description: string | React.ReactElement;
  children: React.ReactNode | React.ReactNode[];
}): React.ReactElement => {
  const classes = useStyles();
  // Grid seems to work based on screen size rather than parent size, so we
  // have to switch our grid sizing based on whether we're in overlay or
  // fullscreen mode
  const { fullscreen } = useContext(ConfigEditorContext);
  const sectionSize: Partial<Record<Breakpoint, GridSize>> = fullscreen
    ? { xs: 12, md: 6 }
    : { xs: 12 };

  return (
    <Grid item {...sectionSize}>
      <section className={classes.configSection} {...rest}>
        <div className={classes.titleWrapper}>
          <Typography className={classes.titleText} variant="h3">
            {title}
          </Typography>
          <HelpTooltip>{description}</HelpTooltip>
        </div>

        {children}
      </section>
    </Grid>
  );
};

export default ConfigSection;
