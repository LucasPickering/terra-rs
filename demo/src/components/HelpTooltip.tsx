import React, { useState } from "react";
import {
  ClickAwayListener,
  IconButton,
  makeStyles,
  Tooltip,
} from "@material-ui/core";
import { HelpOutline as IconHelpOutline } from "@material-ui/icons";

const useStyles = makeStyles(({ typography }) => ({
  tooltipContent: {
    fontSize: typography.body1.fontSize,
  },
}));

const HelpTooltip: React.FC<{
  id?: string;
  children: string | React.ReactElement;
}> = ({ id, children }) => {
  const classes = useStyles();
  const [visible, setVisible] = useState<boolean>(false);

  const toggleOpen = (): void => setVisible((old) => !old);
  const onClose = (): void => setVisible(false);

  return (
    <ClickAwayListener onClickAway={onClose}>
      <Tooltip
        classes={{ tooltip: classes.tooltipContent }}
        id={id}
        open={visible}
        title={children}
        arrow
        interactive
        disableHoverListener
        onClose={onClose}
      >
        <IconButton aria-label="help" size="small" onClick={toggleOpen}>
          <IconHelpOutline />
        </IconButton>
      </Tooltip>
    </ClickAwayListener>
  );
};

export default HelpTooltip;
