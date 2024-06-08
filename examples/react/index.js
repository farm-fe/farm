import {
  createCompiler,
  createDevServer,
  createFileWatcher,
  resolveConfig,
  start,
} from "@farmfe/core";

// const resolvedUserConfig = await resolveConfig({
//   server: {
//     port: 3526,
//   },
//   clearScreen: true,
//   mode: "development",
//   plugins: ["@farmfe/plugin-react", "@farmfe/plugin-sass"],
// });

// const compiler = await createCompiler(resolvedUserConfig);

// const devServer = await createDevServer(compiler, resolvedUserConfig);

// const watcher = await createFileWatcher(devServer, resolvedUserConfig);

// await devServer.listen();

// watcher.watchExtraFiles();

await start({
  server: {
    port: 3526,
  },
  plugins: ["@farmfe/plugin-react", "@farmfe/plugin-sass"],
  compilation: {
    output: {
      publicPath: "/public/",
    }
  }
});
