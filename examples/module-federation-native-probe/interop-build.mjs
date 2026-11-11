import { build } from "@farmfe/core";
import { rspack } from "@rspack/core";
import { ModuleFederationPlugin as RspackFederation } from "@module-federation/enhanced/rspack";
import webpack from "webpack";
import MiniCssExtractPlugin from "mini-css-extract-plugin";
import federation from "@farmfe/plugin-module-federation";
import { configurations } from "./config.mjs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import fs from "node:fs/promises";
const base = path.dirname(fileURLToPath(import.meta.url));
const format = process.env.MF_FORMAT || "global";
const bundler = process.env.MF_BUNDLER || "rspack";
const compile = bundler === "webpack" ? webpack : rspack;
const ModuleFederationPlugin =
  bundler === "webpack"
    ? webpack.container.ModuleFederationPlugin
    : RspackFederation;
const shared = Object.fromEntries(
  ["react", "react/jsx-runtime"].map((key) => [
    key,
    {
      version: "18.3.1",
      requiredVersion: "^18.3.1",
      singleton: true,
      strictVersion: true,
    },
  ]),
);
await build(configurations().find((c) => c.root.endsWith("/remote-a")));
await build({
  root: path.join(base, "interop/farm-host"),
  configFile: false,
  mode: "production",
  compilation: {
    input: { index: "./index.html" },
    output: {
      path: path.join(base, "dist/interop-farm-host"),
      targetEnv: "browser-esnext",
    },
    persistentCache: false,
    lazyCompilation: false,
    sourcemap: false,
    presetEnv: false,
  },
  plugins: [
    federation({
      name: "farmInteropHost",
      shared: Object.fromEntries(
        Object.entries(shared).map(([key, value]) => [
          key,
          { ...value, fallback: fileURLToPath(import.meta.resolve(key)) },
        ]),
      ),
      remotes: {
        rsp: {
          name: "rspProvider",
          globalName: "rspContainer",
          format,
          entry: "http://127.0.0.1:4201/rspack/remoteEntry.js",
        },
      },
    }),
  ],
});
for (const remote of [true, false]) {
  const dir = remote ? "rspack-remote" : "rspack-host";
  const config = {
    mode: "production",
    target: ["web", "es2020"],
    context: path.join(base, "interop", dir),
    entry: remote ? {} : { main: "./index.js" },
    devtool: false,
    output: {
      path: path.join(base, "dist", dir),
      publicPath: remote
        ? "http://127.0.0.1:4201/rspack/"
        : "http://127.0.0.1:4202/",
      filename: "[name].js",
      chunkFilename: "[name].[contenthash].js",
      uniqueName: remote ? "rspProvider" : "rspHost",
      ...(remote && format === "esm" ? { module: true } : {}),
    },
    experiments: {
      ...(bundler === "rspack" ? { css: true } : {}),
      ...(remote && format === "esm" ? { outputModule: true } : {}),
    },
    module: {
      rules: [
        bundler === "webpack"
          ? {
              test: /\.css$/,
              use: [
                MiniCssExtractPlugin.loader,
                fileURLToPath(import.meta.resolve("css-loader")),
              ],
            }
          : { test: /\.css$/, type: "css" },
      ],
    },
    plugins: [
      ...(bundler === "webpack"
        ? [new MiniCssExtractPlugin({ filename: "[name].[contenthash].css" })]
        : []),
      new ModuleFederationPlugin({
        name: remote ? "rspProvider" : "rspHost",
        filename: "remoteEntry.js",
        ...(bundler === "rspack"
          ? { manifest: false, dts: false, shareStrategy: "loaded-first" }
          : {}),
        shared,
        ...(remote
          ? {
              library:
                format === "esm"
                  ? { type: "module" }
                  : { type: "var", name: "rspContainer" },
              exposes: { "./Button": "./Button.js" },
            }
          : {
              remoteType: format === "esm" ? "module" : "script",
              remotes: {
                farm:
                  (format === "esm" ? "" : "remoteA@") +
                  "http://127.0.0.1:4101/deploy/a/remoteEntry.js",
              },
            }),
      }),
    ],
  };
  await new Promise((resolve, reject) => {
    const compiler = compile(config);
    compiler.run((error, stats) => {
      compiler.close((closeError) => {
        if (error || closeError || stats?.hasErrors())
          reject(
            error ||
              closeError ||
              new Error(stats.toString({ all: false, errors: true })),
          );
        else {
          console.log(stats.toString({ all: false, timings: true }));
          resolve();
        }
      });
    });
  });
  if (!remote)
    await fs.writeFile(
      path.join(base, "dist", dir, "index.html"),
      '<!doctype html><div id="app"></div><script src="/main.js"></script>',
    );
}
