export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';

import path from 'node:path';
import fs from 'node:fs/promises';
import os from 'node:os';
import { statSync } from 'node:fs';
import sirv from 'sirv';
import compression from 'koa-compress';
import Koa, { Context } from 'koa';
import fse from 'fs-extra';

import { Compiler } from './compiler/index.js';
import {
  normalizePublicDir,
  resolveConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger, Logger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import { compilerHandler } from './utils/build.js';
import { setProcessEnv } from './config/env.js';
import { colors } from './utils/color.js';
import { useProxy } from './server/middlewares/index.js';

import type { FarmCLIOptions, ResolvedUserConfig } from './config/types.js';
import { JsPlugin } from './plugin/type.js';
import { __FARM_GLOBAL__ } from './config/_global.js';
import { ConfigWatcher } from './watcher/configWatcher.js';
import { clearScreen } from './utils/share.js';

// export async function start(
//   inlineConfig: FarmCLIOptions & UserConfig
// ): Promise<void> {
//   const logger = inlineConfig.logger ?? new DefaultLogger();
//   setProcessEnv('development');
  
//   try {
//     const resolvedUserConfig = await resolveConfig(
//       inlineConfig,
//       logger,
//       'development'
//     );

//     const compiler = await createCompiler(resolvedUserConfig);
//     const devServer = setupDevServer(compiler, resolvedUserConfig, logger);
//     await devServer.listen();

//     setupFileWatcher(devServer, resolvedUserConfig, logger);
//   } catch (error) {
//     logger.error(`Failed to start the server: ${error.message}`);
//     process.exit(1);
//   }
// }


export async function start(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();

  setProcessEnv('development');

  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'development'
  );

  const {
    compilation: compilationConfig,
    server: serverConfig,
    jsPlugins
  } = resolvedUserConfig;

  const compiler = await createCompiler(resolvedUserConfig);

  const devServer = new DevServer(compiler, logger);
  devServer.createFarmServer(serverConfig);

  jsPlugins.forEach((plugin: JsPlugin) =>
    plugin.configureDevServer?.(devServer)
  );

  await devServer.listen();

  let fileWatcher: FileWatcher;
  // Make sure the server is listening before we watch for file changes
  if (devServer.config.hmr) {
    if (compilationConfig.mode === 'production') {
      logger.error(
        'HMR can not be enabled in production mode. Please set the mode option to "development" in your config file.'
      );
      process.exit(1);
    }
    fileWatcher = new FileWatcher(devServer, resolvedUserConfig);
    await fileWatcher.watch();
  }

  const farmWatcher = new ConfigWatcher(resolvedUserConfig).watch(
    (filenames: string[]) => {
      clearScreen();
      logger.info(
        colors.bold(
          colors.green(
            `${filenames
              .map((filename) =>
                path.relative(resolvedUserConfig.root, filename)
              )
              .join(', ')} changed, server will restart.`
          )
        )
      );

      farmWatcher.close();

      devServer.restart(async () => {
        fileWatcher?.close();
        await devServer.closeFarmServer();
        __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
        await start(inlineConfig);
      });
    }
  );
}

export async function build(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('production');
  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'production'
  );

  setProcessEnv(resolvedUserConfig.compilation.mode);

  await createBundleHandler(resolvedUserConfig);

  // copy resources under publicDir to output.path
  await copyPublicDirectory(resolvedUserConfig, inlineConfig, logger);
}

export async function preview(inlineConfig: FarmCLIOptions): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  const port = inlineConfig.port ?? 1911;
  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'production'
  );

  const { root, output } = resolvedUserConfig.compilation;
  const distDir = path.resolve(root, output.path);
  try {
    statSync(distDir);
  } catch (err) {
    if (err.code === 'ENOENT') {
      throw new Error(
        `The directory "${distDir}" does not exist. Did you build your project?`
      );
    }
  }

  function StaticFilesHandler(ctx: Context) {
    const staticFilesServer = sirv(distDir, {
      etag: true,
      single: true
    });
    return new Promise<void>((resolve) => {
      staticFilesServer(ctx.req, ctx.res, () => {
        resolve();
      });
    });
  }
  const app = new Koa();

  // support proxy
  useProxy(resolvedUserConfig.server.proxy, app, logger);

  app.use(compression());
  app.use(async (ctx) => {
    const requestPath = ctx.request.path;

    if (requestPath.startsWith(output.publicPath)) {
      const modifiedPath = requestPath.substring(output.publicPath.length);

      if (modifiedPath.startsWith('/')) {
        ctx.request.path = modifiedPath;
      } else {
        ctx.request.path = `/${modifiedPath}`;
      }
    }
    await StaticFilesHandler(ctx);
  });

  app.listen(port, () => {
    logger.info(colors.green(`preview server running at:\n`));
    const interfaces = os.networkInterfaces();
    Object.keys(interfaces).forEach((key) =>
      (interfaces[key] || [])
        .filter((details) => details.family === 'IPv4')
        .map((detail) => {
          return {
            type: detail.address.includes('127.0.0.1')
              ? 'Local:   '
              : 'Network: ',
            host: detail.address
          };
        })
        .forEach(({ type, host }) => {
          const url = `${'http'}://${host}:${colors.bold(port)}${
            output.publicPath ?? ''
          }`;
          logger.info(`${colors.magenta('>')} ${type} ${colors.cyan(url)}`);
        })
    );
  });
}

export async function watch(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('development');
  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'development'
  );

  setProcessEnv(resolvedUserConfig.compilation.mode);

  const compilerFileWatcher = await createBundleHandler(
    resolvedUserConfig,
    true
  );
  
  async function handleFileChange(files: string[]) {
    const changedFiles = files.map(file => path.relative(resolvedUserConfig.root, file)).join(', ');
    logger.info(
      colors.bold(
        colors.green(
          `${changedFiles} changed, will be restart`
        )
      )
    );

    try {
      farmWatcher.close();

      __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;

      compilerFileWatcher?.close();

      await watch(inlineConfig);
    } catch (error) {
      logger.error(`Error restarting the watcher: ${error.message}`);
    }
  }

  const farmWatcher = new ConfigWatcher(resolvedUserConfig).watch(handleFileChange);
}

export async function clean(
  rootPath: string,
  recursive: boolean | undefined
): Promise<void> {
  // TODO After optimizing the reading of config, put the clean method into compiler
  const logger = new DefaultLogger();

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

export async function createBundleHandler(
  resolvedUserConfig: ResolvedUserConfig,
  watchMode = false
) {
  const compiler = await createCompiler(resolvedUserConfig);

  await compilerHandler(async () => {
    compiler.removeOutputPathDir();
    await compiler.compile();
    compiler.writeResourcesToDisk();
  }, resolvedUserConfig);

  if (resolvedUserConfig.compilation?.watch || watchMode) {
    const watcher = new FileWatcher(compiler, resolvedUserConfig);
    await watcher.watch();
    return watcher;
  }
}

export async function createCompiler(ResolvedUserConfig: ResolvedUserConfig) {
  const {
    jsPlugins,
    rustPlugins,
    compilation: compilationConfig
  } = ResolvedUserConfig;

  const compiler = new Compiler({
    config: compilationConfig,
    jsPlugins,
    rustPlugins
  });

  for (const plugin of jsPlugins) {
    await plugin.configureCompiler?.(compiler);
  }

  return compiler;
}

async function copyPublicDirectory(
  resolvedUserConfig: ResolvedUserConfig,
  inlineConfig: FarmCLIOptions & UserConfig,
  logger: Logger
): Promise<void> {
  const absPublicDirPath = normalizePublicDir(
    resolvedUserConfig.root,
    inlineConfig.publicDir
  );

  try {
    if (await fse.pathExists(absPublicDirPath)) {
      await fse.copy(absPublicDirPath, resolvedUserConfig.compilation.output.path);
      logger.info('Public directory resources copied successfully.');
    }
  } catch (error) {
    logger.error(`Error copying public directory: ${error.message}`);
  }
}

function setupDevServer(compiler, resolvedUserConfig, logger) {
  const devServer = new DevServer(compiler, logger);
  devServer.createFarmServer(resolvedUserConfig.server);

  resolvedUserConfig.jsPlugins.forEach((plugin: JsPlugin) =>
    plugin.configureDevServer?.(devServer)
  );

  return devServer;
}

async function setupFileWatcher(devServer, resolvedUserConfig, logger) {
  if (devServer.config.hmr && resolvedUserConfig.compilation.mode === 'production') {
    logger.error('HMR cannot be enabled in production mode.');
    return;
  }

  const fileWatcher = new FileWatcher(devServer, resolvedUserConfig);
  await fileWatcher.watch();

  const farmWatcher = new ConfigWatcher(resolvedUserConfig).watch(
    async (filenames: string[]) => {
      clearScreen();
      logFileChanges(filenames, resolvedUserConfig.root, logger);

      farmWatcher.close();
      await restartServer(devServer, fileWatcher, inlineConfig);
    }
  );
}

function logFileChanges(filenames, root, logger) {
  const changedFiles = filenames
    .map((filename) => path.relative(root, filename))
    .join(', ');
  logger.info(colors.bold(colors.green(`${changedFiles} changed, server will restart.`)));
}

async function restartServer(devServer, fileWatcher, inlineConfig) {
  fileWatcher?.close();
  await devServer.closeFarmServer();
  __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
  await start(inlineConfig);
}

export { defineFarmConfig as defineConfig } from './config/index.js';



