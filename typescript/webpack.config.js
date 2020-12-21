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
    filename: "[name].bundle.js",
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
    new HtmlWebpackPlugin({
      template: "static/index.html",
      favicon: "static/favicon.ico",
    }),
    new WasmPackPlugin({
      outName: "terra-wasm",
      crateDirectory: wasmDir,
      outDir: path.resolve(wasmDir, "pkg"),
    }),
  ],

  resolve: {
    extensions: [".tsx", ".ts", ".js"],
  },

  optimization: {
    splitChunks: {
      cacheGroups: {
        commons: {
          test: /[\\/]node_modules[\\/]/,
          name: "vendors",
          chunks: "all",
          filename: "[name].app.bundle.js",
        },
      },
    },
  },

  watchOptions: {
    ignored: /world\.json/,
  },
  devServer: {
    port: 3000,
    contentBase: path.join(__dirname, "static"),
    watchContentBase: true,
  },
};
