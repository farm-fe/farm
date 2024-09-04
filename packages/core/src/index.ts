export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';
export type {
  ModuleType,
  ResolveKind,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  OutputConfig,
  ResolveConfig,
  RuntimeConfig,
  ScriptConfig,
  CssConfig,
  PersistentCacheConfig,
  PartialBundlingConfig,
  PresetEnvConfig,
  Config,
  PluginTransformHookParam,
  PluginTransformHookResult
} from './types/binding.js';

import { statSync } from 'node:fs';
import fs from 'node:fs/promises';
import path from 'node:path';
import fse from 'fs-extra';

import { Compiler } from './compiler/index.js';
import { loadEnv, setProcessEnv } from './config/env.js';
import {
  UserConfig,
  checkClearScreen,
  getConfigFilePath,
  normalizePublicDir,
  resolveConfig
} from './config/index.js';
import { Server } from './server/index.js';
import { compilerHandler } from './utils/build.js';
import { colors } from './utils/color.js';
import { Logger } from './utils/logger.js';

import { __FARM_GLOBAL__ } from './config/_global.js';
import type {
  FarmCliOptions,
  ResolvedUserConfig,
  UserPreviewServerConfig
} from './config/types.js';
// import { logError } from './server/error.js';
// import { lazyCompilation } from './server/middlewares/lazy-compilation.js';

import type { JsPlugin } from './plugin/type.js';

// export async function start(
//   inlineConfig?: FarmCliOptions & UserConfig
// ): Promise<void> {
//   inlineConfig = inlineConfig ?? {};
//   const logger = inlineConfig.logger ?? new Logger();
//   setProcessEnv('development');

//   try {
//     const resolvedUserConfig = await resolveConfig(
//       inlineConfig,
//       'start',
//       'development',
//       'development',
//       false
//     );

//     const compiler = await createCompiler(resolvedUserConfig, logger);

//     const devServer = await createDevServer(
//       compiler,
//       resolvedUserConfig,
//       logger
//     );

//     await devServer.listen();
//   } catch (error) {
//     logger.error('Failed to start the server', { exit: true, error });
//   }
// }

// export async function build(
//   inlineConfig?: FarmCliOptions & UserConfig
// ): Promise<void> {
//   inlineConfig = inlineConfig ?? {};
//   const logger = inlineConfig.logger ?? new Logger();
//   setProcessEnv('production');

//   const resolvedUserConfig = await resolveConfig(
//     inlineConfig,
//     'build',
//     'production',
//     'production',
//     false
//   );

//   try {
//     await createBundleHandler(resolvedUserConfig, logger);
//     // copy resources under publicDir to output.path
//     await copyPublicDirectory(resolvedUserConfig, logger);
//   } catch (err) {
//     logger.error(`Failed to build: ${err}`, { exit: true });
//   }
// }

// export async function preview(inlineConfig?: FarmCliOptions): Promise<void> {
//   inlineConfig = inlineConfig ?? {};
//   const logger = inlineConfig.logger ?? new Logger();
//   const resolvedUserConfig = await resolveConfig(
//     inlineConfig,
//     'preview',
//     'production',
//     'production',
//     true
//   );

//   const { root, output } = resolvedUserConfig.compilation;
//   const distDir = path.resolve(root, output.path);

//   try {
//     statSync(distDir);
//   } catch (err) {
//     if (err.code === 'ENOENT') {
//       throw new Error(
//         `The directory "${distDir}" does not exist. Did you build your project?`
//       );
//     }
//   }

//   // reusing port conflict check from DevServer
//   const serverConfig = {
//     ...resolvedUserConfig.server,
//     host: inlineConfig.host ?? true,
//     port:
//       inlineConfig.port ??
//       (Number(process.env.FARM_DEFAULT_SERVER_PORT) || 1911)
//   };
//   await Server.resolvePortConflict(serverConfig, logger);
//   const port = serverConfig.port;
//   const host = serverConfig.host;
//   const previewOptions: UserPreviewServerConfig = {
//     ...serverConfig,
//     distDir,
//     output: { path: output.path, publicPath: output.publicPath },
//     port,
//     host
//   };

//   // const server = new Server({ logger });
//   server.createPreviewServer(previewOptions);
// }

// export async function watch(
//   inlineConfig?: FarmCliOptions & UserConfig
// ): Promise<void> {
//   inlineConfig = inlineConfig ?? {};
//   const logger = inlineConfig.logger ?? new Logger();
//   setProcessEnv('development');

//   inlineConfig.server ??= {};
//   inlineConfig.server.hmr ??= false;

//   const resolvedUserConfig = await resolveConfig(
//     inlineConfig,
//     'build',
//     'production',
//     'production',
//     false
//   );

//   const fileWatcher = await createBundleHandler(
//     resolvedUserConfig,
//     logger,
//     true
//   );

//   let devServer: Server | undefined;
//   // create dev server for lazy compilation
//   const lazyEnabled = resolvedUserConfig.compilation.lazyCompilation;
//   if (lazyEnabled) {
//     devServer = new Server({
//       logger,
//       // TODO type error
//       // @ts-ignore
//       compiler: fileWatcher.serverOrCompiler as Compiler
//     });
//     await devServer.createServer(resolvedUserConfig.server);
//     devServer.applyMiddlewares([lazyCompilation]);
//     await devServer.startServer(resolvedUserConfig.server);
//   }

//   async function handleFileChange(files: string[]) {
//     logFileChanges(files, resolvedUserConfig.root, logger);

//     try {
//       if (lazyEnabled && devServer) {
//         devServer.close();
//       }

//       __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;

//       await fileWatcher?.close();

//       await watch(inlineConfig);
//     } catch (error) {
//       logger.error(`Error restarting the watcher: ${error.message}`);
//     }
//   }

//   // fileWatcher.watchConfigs(handleFileChange);
// }

export async function clean(
  rootPath: string,
  recursive?: boolean | undefined
): Promise<void> {
  // TODO After optimizing the reading of config, put the clean method into compiler
  const logger = new Logger();

  const nodeModulesFolders = recursive
    ? await findNodeModulesRecursively(rootPath)
    : [path.join(rootPath, 'node_modules')];

  await Promise.all(
    nodeModulesFolders.map(async (nodeModulesPath) => {
      // TODO Bug .farm cacheDir folder not right
      const farmFolderPath = path.join(nodeModulesPath, '.farm');
      try {
        const stats = await fs.stat(farmFolderPath);
        if (stats.isDirectory()) {
          await fs.rm(farmFolderPath, { recursive: true, force: true });
          // TODO optimize nodeModulePath path e.g: /Users/xxx/node_modules/.farm/cache
          logger.info(
            `Cache cleaned at ${colors.bold(colors.green(nodeModulesPath))}`
          );
        }
      } catch (error) {
        if (error.code === 'ENOENT') {
          logger.warn(
            `No cached files found in ${colors.bold(
              colors.green(nodeModulesPath)
            )}`
          );
        } else {
          logger.error(
            `Error cleaning cache in ${colors.bold(
              colors.green(nodeModulesPath)
            )}: ${error.message}`
          );
        }
      }
    })
  );
}

async function findNodeModulesRecursively(rootPath: string): Promise<string[]> {
  const result: string[] = [];

  async function traverse(currentPath: string) {
    const items = await fs.readdir(currentPath);
    for (const item of items) {
      const fullPath = path.join(currentPath, item);
      const stats = await fs.stat(fullPath);

      if (stats.isDirectory()) {
        if (item === 'node_modules') {
          result.push(fullPath);
        } else {
          await traverse(fullPath);
        }
      }
    }
  }

  await traverse(rootPath);
  return result;
}

// export async function createBundleHandler(
//   resolvedUserConfig: ResolvedUserConfig,
//   logger: Logger,
//   watchMode = false
// ) {
//   const compiler = await createCompiler(resolvedUserConfig, logger);

//   await compilerHandler(
//     async () => {
//       if (resolvedUserConfig.compilation?.output?.clean) {
//         compiler.removeOutputPathDir();
//       }

//       try {
//         await compiler.compile();
//       } catch (err) {
//         // throw new Error(logError(err) as unknown as string);
//         throw new Error(err as unknown as string);
//       }
//       compiler.writeResourcesToDisk();
//     },
//     resolvedUserConfig,
//     logger
//   );

//   if (resolvedUserConfig.compilation?.watch || watchMode) {
//     const watcher = new FileWatcher(compiler, resolvedUserConfig, logger);
//     await watcher.watch();
//     return watcher;
//   }
// }

export async function createCompiler(
  resolvedUserConfig: ResolvedUserConfig,
  logger: Logger
) {
  const compiler = initInlineCompiler(resolvedUserConfig, logger);
  // TODO 这也不对 这块要 先排序 然后过滤掉对应的 存在这个钩子的插件 然后进而调用一些 like e.g 中间件
  // 这块逻辑也需要过滤 拿到最新的
  for (const plugin of resolvedUserConfig.jsPlugins) {
    await plugin.configureCompiler?.(compiler);
  }

  return compiler;
}

export function initInlineCompiler(
  resolvedUserConfig: ResolvedUserConfig,
  logger: Logger
) {
  const {
    jsPlugins,
    rustPlugins,
    compilation: compilationConfig
  } = resolvedUserConfig;

  const compiler = new Compiler(
    {
      config: compilationConfig,
      jsPlugins,
      rustPlugins
    },
    logger
  );
  return compiler;
}

export async function createInlineCompiler(
  config: ResolvedUserConfig,
  options = {}
) {
  const { Compiler } = await import('./compiler/index.js');
  return new Compiler({
    config: { ...config.compilation, ...options },
    jsPlugins: config.jsPlugins,
    rustPlugins: config.rustPlugins
  });
}

async function copyPublicDirectory(
  resolvedUserConfig: ResolvedUserConfig,
  logger: Logger
): Promise<void> {
  const absPublicDirPath = normalizePublicDir(
    resolvedUserConfig.root,
    resolvedUserConfig.publicDir
  );

  try {
    if (await fse.pathExists(absPublicDirPath)) {
      const files = await fse.readdir(absPublicDirPath);
      const outputPath = resolvedUserConfig.compilation.output.path;
      for (const file of files) {
        const publicFile = path.join(absPublicDirPath, file);
        const destFile = path.join(outputPath, file);

        if (await fse.pathExists(destFile)) {
          continue;
        }
        await fse.copy(publicFile, destFile);
      }

      logger.info(
        `Public directory resources copied ${colors.bold(
          colors.green('successfully')
        )}.`
      );
    }
  } catch (error) {
    logger.error(`Error copying public directory: ${error.message}`);
  }
}

// export async function createDevServer(
//   compiler: Compiler,
//   resolvedUserConfig: ResolvedUserConfig,
//   logger: Logger
// ) {
//   const server = new Server({ compiler, logger });
//   await server.createDevServer(resolvedUserConfig.server);
//   await createFileWatcher(server, resolvedUserConfig, logger);
//   // call configureDevServer hook after both server and watcher are ready
//   resolvedUserConfig.jsPlugins.forEach((plugin: JsPlugin) =>
//     plugin.configureDevServer?.(server)
//   );

//   return server;
// }

// export async function createFileWatcher(
//   devServer: Server,
//   resolvedUserConfig: ResolvedUserConfig,
//   logger: Logger = new Logger()
// ) {
// if (
//   devServer.config.hmr &&
//   resolvedUserConfig.compilation.mode === "production"
// ) {
//   logger.error("HMR cannot be enabled in production mode.");
//   return;
// }

//   if (!devServer.config.hmr) {
//     return;
//   }

//   if (devServer.watcher) {
//     return;
//   }

//   const configFilePath = await getConfigFilePath(resolvedUserConfig.root);
//   const fileWatcher = new FileWatcher(
//     // @ts-ignore
//     devServer,
//     { ...resolvedUserConfig, configFilePath },
//     logger
//   );
//   devServer.watcher = fileWatcher;
//   await fileWatcher.watch();

//   // fileWatcher.watchConfigs(async (files: string[]) => {
//   //   checkClearScreen(resolvedUserConfig);

//   //   devServer.restart(async () => {
//   //     logFileChanges(files, resolvedUserConfig.root, logger);
//   //     fileWatcher?.close();

//   //     await devServer.close();
//   //     __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
//   //     await start(resolvedUserConfig as FarmCliOptions & UserConfig);
//   //   });
//   // });
//   return fileWatcher;
// }

// export async function createFileWatcher2(
//   devServer: Server,
//   resolvedUserConfig: ResolvedUserConfig,
//   logger: Logger = new Logger()
// ) {
//   // if (
//   //   resolvedUserConfig.server.hmr &&
//   //   resolvedUserConfig.compilation.mode === "production"
//   // ) {
//   //   logger.error("HMR cannot be enabled in production mode.");
//   //   return;
//   // }

//   if (!resolvedUserConfig.server.hmr) {
//     return;
//   }

//   // if (resolvedUserConfig.server.watcher) {
//   //   return;
//   // }

//   const configFilePath = await getConfigFilePath(resolvedUserConfig.root);
//   // const fileWatcher = new FileWatcher(
//   //   devServer,
//   //   { ...resolvedUserConfig, configFilePath },
//   //   logger,
//   // );
//   // devServer.watcher = fileWatcher;
//   // await fileWatcher.watch();

//   // fileWatcher.watchConfigs(async (files: string[]) => {
//   //   checkClearScreen(resolvedUserConfig);
//   //   logFileChanges(files, resolvedUserConfig.root, logger);

//   //   await fileWatcher.close();
//   //   await devServer.close();
//   //   __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;

//   //   await start2(resolvedUserConfig as FarmCliOptions & UserConfig);
//   //   // devServer.restart(async () => {
//   //   //   logFileChanges(files, resolvedUserConfig.root, logger);
//   //   //   farmWatcher?.close();

//   //   //   await devServer.close();
//   //   //   __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
//   //   //   await start(resolvedUserConfig as FarmCliOptions & UserConfig);
//   //   // });
//   // });
//   // return fileWatcher;
// }

export function logFileChanges(files: string[], root: string, logger: Logger) {
  const changedFiles = files
    .map((file) => path.relative(root, file))
    .join(', ');
  logger.info(
    colors.bold(colors.green(`${changedFiles} changed, server will restart.`))
  );
}

export { defineFarmConfig as defineConfig } from './config/index.js';

export { loadEnv, Server };

export async function start(
  inlineConfig?: FarmCliOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  const logger = inlineConfig.logger ?? new Logger();
  setProcessEnv('development');

  try {
    const resolvedUserConfig = await resolveConfig(
      inlineConfig,
      'start',
      'development',
      'development',
      false
    );

    const server = new Server(resolvedUserConfig, logger);

    await server.createServer();
    // TODO 这段逻辑放在 创建 http server 之后 放到  server 里面
    resolvedUserConfig.jsPlugins.forEach((plugin: JsPlugin) =>
      // @ts-ignore
      plugin.configureDevServer?.(server)
    );
    server.listen();
  } catch (error) {
    logger.error('Failed to start the server', { exit: true, error });
  }
}
