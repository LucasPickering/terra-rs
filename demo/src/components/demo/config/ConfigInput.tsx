import React, { useContext } from "react";
import { makeStyles } from "@material-ui/core";
import DemoContext from "context/DemoContext";
import { get, Path } from "../../../util";
import HelpTooltip from "components/HelpTooltip";
import { WorldConfigObject } from "terra-wasm";

const useStyles = makeStyles(({ spacing }) => ({
  configInputWrapper: {
    marginTop: spacing(1),
  },
}));

interface Props<T> {
  field: Path<WorldConfigObject>;
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
  const { worldConfigHandler } = useContext(DemoContext);
  const value: T = get(worldConfigHandler.config, field);
  const onChange = (value: T): void => {
    worldConfigHandler.setField(field, value);
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
