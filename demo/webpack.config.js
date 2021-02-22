const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const { BundleAnalyzerPlugin } = require("webpack-bundle-analyzer");

const coreCrateDir = path.resolve(__dirname, "../crates/core");
const wasmCrateDir = path.resolve(__dirname, "../crates/wasm");

module.exports = {
  mode: process.env.NODE_ENV || "development",
  entry: "./src/index.tsx",
  target: "web",
  output: {
    path: path.resolve(__dirname, "dist"),
    publicPath: "/",
    filename: "[name].bundle.js",
  },

  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node_modules/,
      },
      {
        test: /\.wasm$/,
        include: path.resolve(__dirname, "src"),
        use: "wasm-loader",
      },
      {
        test: /\.css$/i,
        use: ["style-loader", "css-loader"],
      },
      {
        test: /\.js$/,
        enforce: "pre",
        use: ["source-map-loader"],
      },
    ],
  },

  experiments: {
    syncWebAssembly: true,
    topLevelAwait: true,
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: "public/index.html",
      favicon: "public/favicon.ico",
    }),
    new WasmPackPlugin({
      outName: "terra-wasm",
      crateDirectory: wasmCrateDir,
      watchDirectories: [
        path.resolve(coreCrateDir, "Cargo.toml"),
        path.resolve(coreCrateDir, "src"),
        path.resolve(wasmCrateDir, "Cargo.toml"),
        path.resolve(wasmCrateDir, "src"),
      ],
      outDir: path.resolve(wasmCrateDir, "pkg"),
    }),
    new BundleAnalyzerPlugin({
      analyzerMode: process.env.WEBPACK_BUNDLE_ANALYZER_MODE || "disabled",
    }),
  ],

  resolve: {
    modules: [path.resolve(__dirname, "src"), "node_modules"],
    extensions: [".tsx", ".ts", ".js"],
  },

  optimization: {
    splitChunks: {
      cacheGroups: {
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          priority: -20,
          name: "vendors",
          chunks: "all",
          filename: "[name].app.bundle.js",
        },
        babylonjs: {
          test: /[\\/]node_modules[\\/]@babylonjs[\\/]/,
          priority: -10,
          name: "vendors-babylonjs",
          chunks: "all",
          filename: "[name].app.bundle.js",
        },
      },
    },
  },

  watchOptions: {
    ignored: /node_modules/,
  },
  devServer: {
    port: 3000,
    contentBase: path.join(__dirname, "public"),
    historyApiFallback: true,
    watchContentBase: true,
    hot: true,
  },
};
