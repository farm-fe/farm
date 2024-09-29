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

import { loadEnv, setProcessEnv } from './config/env.js';
import {
  UserConfig,
  normalizePublicDir,
  resolveConfig
} from './config/index.js';
import { Server } from './server/index.js';
import { PersistentCacheBrand, bold, colors, green } from './utils/color.js';
import { Logger } from './utils/logger.js';

import {
  createCompiler,
  resolveConfigureCompilerHook
} from './compiler/utils.js';
import { __FARM_GLOBAL__ } from './config/_global.js';
import type { FarmCliOptions, ResolvedUserConfig } from './config/types.js';
import { getShortName } from './utils/path.js';

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
  resolvedUserConfig: ResolvedUserConfig
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

      resolvedUserConfig.logger.info(
        `Public directory resources copied ${colors.bold(
          colors.green('successfully')
        )}.`
      );
    }
  } catch (error) {
    resolvedUserConfig.logger.error(
      `Error copying public directory: ${error.message}`
    );
  }
}

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
  setProcessEnv('development');
  const server = new Server(inlineConfig);
  try {
    await server.createServer();

    server.listen();
  } catch (error) {
    server.logger.error('Failed to start the server', { exit: false, error });
  }
}

export async function build(
  inlineConfig?: FarmCliOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  setProcessEnv('production');

  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    'build',
    'production',
    'production'
  );

  const { persistentCache, output } = resolvedUserConfig.compilation;

  try {
    const compiler = await createCompiler(resolvedUserConfig);
    await resolveConfigureCompilerHook(compiler, resolvedUserConfig);

    if (output?.clean) {
      compiler.removeOutputPathDir();
    }
    const startTime = performance.now();
    await compiler.compile();
    const elapsedTime = Math.floor(performance.now() - startTime);
    const persistentCacheText = persistentCache
      ? bold(PersistentCacheBrand)
      : '';

    const shortFile = getShortName(
      resolvedUserConfig.configFilePath,
      resolvedUserConfig.root
    );
    resolvedUserConfig.logger.info(
      `Using config file at ${bold(green(shortFile))}`
    );
    resolvedUserConfig.logger.info(
      `Build completed in ${bold(
        green(`${elapsedTime}ms`)
      )} ${persistentCacheText} Resources emitted to ${bold(green(output.path))}.`
    );
    compiler.writeResourcesToDisk();
    await copyPublicDirectory(resolvedUserConfig);
  } catch (err) {
    resolvedUserConfig.logger.error(`Failed to build: ${err}`, { exit: true });
  }
}

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
