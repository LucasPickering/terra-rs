import React from "react";
import { Slider } from "@material-ui/core";

type Props = React.ComponentProps<typeof Slider> & {
  value?: number;
  markValues?: number[];
  formatMark?: (value: number) => string;
  onChange?: (value: number) => void;
};

const RangeConfigInput: React.FC<Props> = ({
  value,
  min = 0,
  max = 10,
  markValues,
  step,
  formatMark = (v) => v.toString(),
  onChange,
  ...rest
}) => {
  let marks: Array<{ value: number; label?: string }>;
  if (markValues) {
    marks = markValues.map((value) => ({ value }));
  } else {
    // Add evenly spaced marks based on the min/step/max
    marks = [{ value: min }];
    const stepNum = step ?? 1;
    for (let i = min + stepNum; i < max; i += stepNum) {
      marks.push({ value: i });
    }
    marks.push({ value: max });
  }

  // Add a label to the first and last marks
  marks[0].label = formatMark(marks[0].value);
  marks[marks.length - 1].label = formatMark(marks[marks.length - 1].value);

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
