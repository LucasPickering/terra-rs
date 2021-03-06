import { useEffect, useState, useRef } from "react";

/**
 * Debounces an input value, so that any changes to it aren't reflected in the
 * output until after a delay. Any changes that occur during the delay will
 * reset the delay. The new output value won't appear until after the until
 * value hasn't changed for the defined wait time, at which point the most
 * recent input value will become the output.
 * @param value The current value
 * @param wait The amount of milliseconds to wait after a value change before updating the output
 * @return The debounced value
 */
const useDebouncedValue = <T>(value: T, wait: number): T => {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);
  const timeoutRef = useRef<number | undefined>();

  useEffect(() => {
    if (timeoutRef.current !== undefined) {
      window.clearTimeout(timeoutRef.current);
    }
    timeoutRef.current = window.setTimeout(() => {
      setDebouncedValue(value);
    }, wait);
  }, [value, wait]);

  return debouncedValue;
};

export default useDebouncedValue;
