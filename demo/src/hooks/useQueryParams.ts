import { useCallback, useMemo } from "react";
import { useHistory } from "react-router-dom";

/**
 * Parse the query params in the current route and return them.
 */
const useQueryParams = (): {
  params: URLSearchParams;
  update: (newParams: Record<string, unknown>) => void;
} => {
  const history = useHistory();
  const search = history.location.search;
  const params = useMemo(() => new URLSearchParams(search), [search]);
  const update = useCallback(
    (newParams: Record<string, unknown>) => {
      Object.entries(newParams).forEach(([key, value]) => {
        params.set(key, JSON.stringify(value));
      });
      history.replace({
        ...history.location,
        search: params.toString(),
      });
    },
    [history, params]
  );

  return {
    params,
    update,
  };
};

export default useQueryParams;
