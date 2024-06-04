import {
  createCompiler,
  createDevServer,
  createFileWatcher,
  resolveConfig,
  start
} from "@farmfe/core";

// const resolvedUserConfig = await resolveConfig();

// const compiler = await createCompiler(resolvedUserConfig);

// const devServer = await createDevServer(compiler, resolvedUserConfig);

// const watcher = await createFileWatcher(devServer, resolvedUserConfig);

// await devServer.listen();
// watcher.watchExtraFiles();


await start()
