const path = require("path");
const webpack = require("webpack");
const HtmlWebpackPlugin = require("html-webpack-plugin");
// const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const wasmDir = path.resolve(__dirname, "../rust");

module.exports = {
  mode: "development",
  entry: "./src/index.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node_modules/,
      },
    ],
  },
  experiments: {
    syncWebAssembly: true,
    topLevelAwait: true,
  },
  plugins: [
    new HtmlWebpackPlugin({ template: "static/index.html" }),
    new WasmPackPlugin({
      outName: "terra",
      crateDirectory: wasmDir,
      outDir: path.resolve(wasmDir, "pkg"),
      forceMode: "profiling",
    }),
  ],
  resolve: {
    extensions: [".tsx", ".ts", ".js"],
  },
  devServer: {
    port: 3000,
    contentBase: path.join(__dirname, "static"),
    watchContentBase: true,
  },
};
