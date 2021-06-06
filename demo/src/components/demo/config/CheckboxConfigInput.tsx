import React from "react";
import { Checkbox } from "@material-ui/core";

type Props = React.ComponentProps<typeof Checkbox> & {
  value?: boolean;
  onChange?: (value: boolean) => void;
};

const CheckboxConfigInput: React.FC<Props> = ({ value, onChange, ...rest }) => (
  <div>
    <Checkbox
      color="primary"
      checked={value}
      onChange={onChange && ((e) => onChange(e.target.checked))}
      {...rest}
    />
  </div>
);

export default CheckboxConfigInput;
