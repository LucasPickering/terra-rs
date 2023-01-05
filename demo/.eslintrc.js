module.exports = {
  env: {
    browser: true,
    es6: true,
  },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    project: "./tsconfig.json",
  },
  plugins: ["@typescript-eslint", "prettier"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:prettier/recommended",
    "plugin:react/recommended",
    "plugin:react-hooks/recommended",
  ],
  globals: {
    Atomics: "readonly",
    SharedArrayBuffer: "readonly",
  },
  settings: {
    react: {
      version: "detect",
    },
  },
  rules: {
    "no-restricted-syntax": [
      "error",
      {
        // Plain import statements on wasm imports "work", but they create weird
        // issues with webpack and force us to use React Suspense, so best to
        // just avoid them
        selector: 'ImportDeclaration[importKind="value"][source.value="terra"]',
        message:
          "Use `import type` or `const ... = await import(...)` for Wasm imports",
      },
    ],

    "no-console": "warn",
    "no-unused-vars": "off", // use the TS rule

    "@typescript-eslint/no-unused-vars": "error",
    "@typescript-eslint/no-explicit-any": ["error", { fixToUnknown: true }],
    "@typescript-eslint/explicit-function-return-type": [
      "error",
      { allowExpressions: true, allowTypedFunctionExpressions: true },
    ],
    "@typescript-eslint/no-object-literal-type-assertion": "off",
    "@typescript-eslint/no-inferrable-types": [
      "error",
      { ignoreParameters: true },
    ],
    "@typescript-eslint/camelcase": "off", // we use names from Rust

    "react/prop-types": "off",
    "react/no-unescaped-entities": [
      "error",
      {
        forbid: [
          {
            char: "<",
            alternatives: ["&lt;"],
          },
          {
            char: ">",
            alternatives: ["&gt;"],
          },
          {
            char: "}",
            alternatives: ["&#125;"],
          },
        ],
      },
    ],
    "react-hooks/exhaustive-deps": "error",
  },
};
