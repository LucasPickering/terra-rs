import { Color4 } from "@babylonjs/core";

type PathTree<T> = {
  [P in keyof T]: T[P] extends Record<string, unknown>
    ? [P, ...Path<T[P]>]
    : [P];
};

/**
 * A path into a nested object. Ripped from https://stackoverflow.com/a/65963590/1907353
 */
export type Path<T> = PathTree<T>[keyof PathTree<T>];

/**
 * A type-hacked function to get a nested value in an object using an array of
 * keys. Each element in the array will grab the next level of nested value.
 * It's theoretically possible to properly type this but it's a giant pain in
 * the ass so not worth it.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any,@typescript-eslint/explicit-module-boundary-types
export function get(obj: any, path: string[]): any {
  if (path.length === 0) {
    throw new Error("Cannot get value for empty path");
  }

  const field = path[0];

  if (path.length === 1) {
    return obj[field];
  }

  return get(obj[field], path.slice(1));
}

/**
 * A type-hacked function to set a nested value in an object using an array of
 * keys. Each element in the array will grab the next level of nested value.
 * It's theoretically possible to properly type this but it's a giant pain in
 * the ass so not worth it.
 */
// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types,@typescript-eslint/no-explicit-any
export function set(obj: any, path: string[], value: unknown): void {
  if (path.length === 0) {
    throw new Error("Cannot set value for empty key");
  }

  const field = path[0];

  if (path.length === 1) {
    obj[field] = value;
  } else {
    set(obj[path[0]], path.slice(1), value);
  }
}

/**
 * Assert that a point in the code is unreachable. Useful for making sure switch
 * statements are exhaustive.
 * @param x The value that should never occur
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function assertUnreachable(x: never): never {
  throw new Error("Didn't expect to get here");
}

export function hexCodeToColor4(hexCode: string): Color4 {
  const matches = hexCode.match(
    /^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/
  );
  if (!matches) {
    throw new Error(`Invalid hex code: ${hexCode}`);
  }

  const [, red, green, blue] = matches;
  return new Color4(
    parseInt(red, 16) / 0xff,
    parseInt(green, 16) / 0xff,
    parseInt(blue, 16) / 0xff,
    1.0
  );
}

export function debounce(f: () => unknown, delay: number): () => void {
  const timeout: { id: number | undefined } = { id: undefined };

  return () => {
    if (timeout.id !== undefined) {
      window.clearTimeout(timeout.id);
    }
    timeout.id = window.setTimeout(f, delay);
  };
}
