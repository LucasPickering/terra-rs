import { useRef } from "react";

/**
 * Initialize a static value once, then reference it forever!
 */
function useStaticValue<T>(init: () => T): T {
  const ref = useRef<{ initialized: boolean; value: T | undefined }>({
    initialized: false,
    value: undefined,
  });
  if (!ref.current.initialized) {
    ref.current = { initialized: true, value: init() };
  }
  return ref.current.value as T;
}

export default useStaticValue;
