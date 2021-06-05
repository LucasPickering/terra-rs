import React from "react";
import { makeStyles } from "@material-ui/core";
import { get, Path } from "../../../util";
import HelpTooltip from "components/HelpTooltip";
import { ConfigHandler } from "hooks/useConfigHandler";

const useStyles = makeStyles(({ spacing }) => ({
  configInputWrapper: {
    marginTop: spacing(1),
  },
}));

interface Props<C> {
  configHandler: ConfigHandler<C>;
  field: Path<C>;
  label: string;
  description: string | React.ReactElement;
  children: React.ReactElement<{
    id: string;
    value: unknown;
    onChange: (value: unknown) => void;
  }>;
}

/**
 * A wrapper component for a user input that modifies a single config field.
 * The type of the config being modified is generic. The given path is
 * enforced to be a valid path into the config, but the value at that path is
 * not enforced to be the same as what the children of this input takes/spits
 * out. I got lazy on the typing, sorry.
 *
 * We know this is living inside a ConfigEditor so it'd be nice to be able to
 * pull the config handler from context, but generic context doesn't reall work
 * so we would have to sacrifice the type safety on the field for that, which
 * isn't worth it. So we just have to pass the config handler into each input.
 */
function ConfigInput<C>({
  configHandler,
  field,
  label,
  description,
  children,
}: Props<C>): React.ReactElement {
  const classes = useStyles();
  const id = `input-${field}`;
  const tooltipId = `tooltip-${field}`;
  const value = get(configHandler.config, field);
  const onChange = (value: unknown): void => {
    configHandler.setField(field, value);
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
