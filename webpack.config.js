const path = require("path");
const webpack = require("webpack");
const HtmlWebpackPlugin = require("html-webpack-plugin");
// const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  mode: "development",
  entry: "./dom/index.ts",
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
      crateDirectory: path.resolve(__dirname, "."),
      outDir: path.resolve(__dirname, "pkg"),
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
