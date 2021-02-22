// type PathsToStringProps<T> = T extends string
//   ? []
//   : {
//       [K in Extract<keyof T, string>]: [K, ...PathsToStringProps<T[K]>];
//     }[Extract<keyof T, string>];

// type Join<T extends string[], D extends string> = T extends []
//   ? never
//   : T extends [infer F]
//   ? F
//   : T extends [infer F, ...infer R]
//   ? F extends string
//     ? `${F}${D}${Join<Extract<R, string[]>, D>}`
//     : never
//   : string;

// /**
//  * Type-safe dotted paths into the config. Ripped from
//  * https://stackoverflow.com/a/47058976/1907353
//  */
// export type ConfigPath = Join<
//   PathsToStringProps<typeof WorldConfigObject>,
//   "."
// >;
