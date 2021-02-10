import React, { useContext } from "react";
import { makeStyles } from "@material-ui/core";
import DemoContext, { ConfigKey } from "context/DemoContext";
import { get } from "../../util";
import HelpTooltip from "../HelpTooltip";

const useStyles = makeStyles(({ spacing }) => ({
  configInputWrapper: {
    marginTop: spacing(1),
  },
}));

interface Props<T> {
  field: ConfigKey;
  label: string;
  description: string | React.ReactElement;
  children: React.ReactElement<{
    id: string;
    value: T;
    onChange: (value: T) => void;
  }>;
}

function ConfigInput<T>({
  field,
  label,
  description,
  children,
}: Props<T>): React.ReactElement {
  const classes = useStyles();
  const id = `input-${field}`;
  const tooltipId = `tooltip-${field}`;
  const { config, setConfigValue } = useContext(DemoContext);
  const value: T = get(config, field);
  const onChange = (value: T): void => {
    setConfigValue(field, value);
  };

  return (
    <div className={classes.configInputWrapper}>
      <label htmlFor={id} aria-describedby={tooltipId}>
        {label}
        <HelpTooltip id={tooltipId}>{description}</HelpTooltip>
      </label>

      {React.cloneElement(children, { id, value, onChange })}
    </div>
  );
}

export default ConfigInput;
