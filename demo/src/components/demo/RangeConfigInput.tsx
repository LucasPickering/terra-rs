import React from "react";
import { Slider } from "@material-ui/core";

type Props = React.ComponentProps<typeof Slider> & {
  value?: number;
  formatMark?: (value: number) => string;
  onChange?: (value: number) => void;
};

const RangeConfigInput: React.FC<Props> = ({
  value,
  min = 0,
  max = 10,
  step,
  formatMark = (v) => v.toString(),
  onChange,
  ...rest
}) => {
  const stepNum = step ?? 1;

  const marks: Array<{ value: number; label?: string }> = [
    { value: min, label: formatMark(min) },
  ];
  for (let i = min + stepNum; i < max; i += stepNum) {
    marks.push({ value: i });
  }
  marks.push({ value: max, label: formatMark(max) });

  return (
    <Slider
      value={value}
      min={min}
      max={max}
      step={step}
      marks={marks}
      color="primary"
      valueLabelDisplay="auto"
      onChange={
        onChange &&
        ((e, value) => onChange(Array.isArray(value) ? value[0] : value))
      }
      {...rest}
    />
  );
};

export default RangeConfigInput;
