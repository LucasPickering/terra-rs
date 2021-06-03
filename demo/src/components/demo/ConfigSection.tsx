import React from "react";
import { makeStyles, Typography } from "@material-ui/core";
import HelpTooltip from "../HelpTooltip";

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

  return (
    <section className={classes.configSection} {...rest}>
      <div className={classes.titleWrapper}>
        <Typography className={classes.titleText} variant="h3">
          {title}
        </Typography>
        <HelpTooltip>{description}</HelpTooltip>
      </div>

      {children}
    </section>
  );
};

export default ConfigSection;
