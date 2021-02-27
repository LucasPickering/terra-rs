import React from "react";
import { makeStyles, Select } from "@material-ui/core";

type Props = React.ComponentProps<typeof Select> & {
  value?: string;
  onChange?: (value: string) => void;
};

const useStyles = makeStyles({
  selectConfigInput: {
    display: "flex",
  },
});

const SelectConfigInput: React.FC<Props> = ({
  value,
  onChange,
  children,
  ...rest
}) => {
  const classes = useStyles();
  return (
    <Select
      className={classes.selectConfigInput}
      value={value}
      color="primary"
      onChange={onChange && ((e) => onChange(e.target.value as string))}
      {...rest}
    >
      {children}
    </Select>
  );
};

export default SelectConfigInput;
