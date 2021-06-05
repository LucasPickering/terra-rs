import { makeStyles, TextField } from "@material-ui/core";
import React from "react";

type Props = React.ComponentProps<typeof TextField> & {
  value?: string;
  onChange?: (value: string) => void;
};

const useStyles = makeStyles(() => ({
  textField: {
    width: "100%",
  },
}));

const TextConfigInput: React.FC<Props> = ({ value, onChange, ...rest }) => {
  const classes = useStyles();
  return (
    <TextField
      className={classes.textField}
      color="primary"
      variant="outlined"
      size="small"
      value={value}
      onChange={onChange && ((e) => onChange(e.target.value))}
      {...rest}
    />
  );
};

export default TextConfigInput;
