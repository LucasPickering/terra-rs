import { useState } from "react";
import { useHistory } from "react-router";
import { Path, set } from "../util";
import useQueryParams from "./useQueryParams";

/**
 * Return value of the config handler hook. This provides any functionality
 * you might need to interact with the config object.
 */
export interface ConfigHandler<T> {
  /**
   * The config object, as an interface type
   */
  config: T;

  /**
   * Modify a single value on the config object. Key should be a string array
   * defining which value to modify.
   */
  setField: (key: Path<T>, value: unknown) => void;

  /**
   * Reset the config value to default
   */
  reset: (confirm: boolean) => void;

  /**
   * Parse the given JSON into a config, and validate it. This will also
   * populate defaults. Returns the parsed and validated config. Throws if
   * the JSON fails to parse or validate.
   */
  validate: (json: string) => T;

  /**
   * Parse and validate a config value out of the given JSON, and replace the
   * current config  with the parsed value. This will also populate defaults as
   * needed. Throws if the JSON fails to parse or validate.
   */
  setFromJson: (json: string) => void;

  /**
   * Write the current config value to the pre-defined query param. Will throw
   * if no query param is defined for this config.
   */
  updateQueryParam: () => void;
}

/**
 * A hook that manages interactions around a particular configuration object.
 * A config object is assume to be a TS interface with an arbitrary number of
 * fields, some of which could be nested. The config interface can also be
 * validated using Rust functions. Validation also populates the defaults of
 * a config.
 *
 * This handler can be used to parse JSON strings into configs, validate, set
 * individual values, and persist the config to the URL via a query param.
 *
 * @param validator A function that validates an arbitrary JS value as a
 *  config object, and populates defaults along the way. If called with an
 *  invalid config value, it should throw
 * @param queryParam If provided, the config will be persisted to this query
 *  param in the URL. On first load, the config will be loaded from this
 *  param, and it can be manually written back using the updateQueryParam
 *  function in the return value
 */
// Yes we really want `object`, it doesn't work with Record
// eslint-disable-next-line @typescript-eslint/ban-types
export function useConfigHandler<T extends object>({
  validator,
  queryParam,
}: {
  validator: (input: unknown) => T;
  queryParam?: string;
}): ConfigHandler<T> {
  const history = useHistory();
  const { params: queryParams } = useQueryParams();

  // Store the config as a **JS object**. We'll update this every time the user
  // makes changes, then convert it back into a Wasm value exactly when it's
  // needed.
  const [config, setConfig] = useState<T>(() => {
    // If the caller specified a query param to use for persisting the config,
    // then we should read the initial value from there (if present)
    if (queryParam) {
      // If there's a config object in the URL query, use that. If not (or if
      // parsing the query fails), fall back to the default.
      const queryConfigStr = queryParams.get(queryParam);
      if (queryConfigStr) {
        try {
          const queryConfigObj = JSON.parse(queryConfigStr);
          // Make sure this is a valid config. If not, this will throw.
          // This will also populate defaults where missing
          return validator(queryConfigObj);
        } catch (e) {
          // eslint-disable-next-line no-console
          console.warn("Error parsing config from query params:", e);
        }
      }
    }

    // Query param not supported or not present, just use the default value
    // Validation populates all defaults, so validating an empty object will
    // just give us the default value.
    return validator({});
  });

  const setField = (key: Path<T>, value: unknown): void => {
    // Shallow copy to force a re-render from React
    const newConfig = { ...config };
    set(newConfig, key, value);
    setConfig(newConfig);
  };
  const reset = (confirm: boolean): void => {
    // If requested, show a window.confirm box before resetting
    if (
      !confirm ||
      window.confirm("Are you sure? You will lose all your current settings.")
    ) {
      // Reset to default values
      setConfig(validator({}));
    }
  };
  const validate = (json: string): T => {
    // If the config is malformed or invalid, this will throw!!
    const parsed = JSON.parse(json);
    return validator(parsed);
  };
  const setFromJson = (json: string): void => {
    // If the config is malformed or invalid, this will throw!!
    setConfig(validate(json));
  };
  const updateQueryParam = (): void => {
    if (!queryParam) {
      throw new Error("Cannot update URL: No query param set for this config");
    }

    // Update the query param
    const newParams = new URLSearchParams();
    newParams.set(queryParam, JSON.stringify(config));
    const search = newParams.toString();
    history.replace({ ...history.location, search });
  };

  return { config, setField, reset, setFromJson, validate, updateQueryParam };
}
