/**
 * Assert that a point in the code is unreachable. Useful for making sure switch
 * statements are exhaustive.
 * @param x The value that should never occur
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function assertUnreachable(x: never): never {
  throw new Error("Didn't expect to get here");
}
