export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';

import path from 'node:path';
import fs from 'node:fs/promises';
import os from 'node:os';
import { existsSync, statSync } from 'node:fs';
import sirv from 'sirv';
import compression from 'koa-compress';
import Koa, { Context } from 'koa';
import fse from 'fs-extra';

import { Compiler } from './compiler/index.js';
import {
  normalizeDevServerOptions,
  normalizePublicDir,
  normalizeUserCompilationConfig,
  resolveUserConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import { Config } from '../binding/index.js';
import { compilerHandler } from './utils/build.js';
import { setProcessEnv } from './config/env.js';
import { bold, cyan, green, magenta } from './utils/color.js';
import { useProxy } from './server/middlewares/index.js';

import type { FarmCLIOptions } from './config/types.js';
import { JsPlugin } from './plugin/type.js';
import { __FARM_GLOBAL__ } from './config/_global.js';
import { ConfigWatcher } from './watcher/configWatcher.js';

export async function start(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();

  setProcessEnv('development');
  const config: UserConfig = await resolveUserConfig(
    inlineConfig,
    'serve',
    logger
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    inlineConfig,
    config,
    logger
  );

  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, config, normalizedConfig);
  devServer.createFarmServer(devServer.userConfig.server);

  if (normalizedConfig.config.mode === 'development') {
    normalizedConfig.jsPlugins.forEach((plugin: JsPlugin) =>
      plugin.configDevServer?.(devServer)
    );
  }

  await devServer.listen();

  let fileWatcher: FileWatcher;
  // Make sure the server is listening before we watch for file changes
  if (devServer.config.hmr) {
    if (normalizedConfig.config.mode === 'production') {
      logger.error(
        'HMR can not be enabled in production mode. Please set the mode option to "development" in your config file.'
      );
      process.exit(1);
    }
    fileWatcher = new FileWatcher(devServer, {
      ...normalizedConfig,
      ...config
    });
    await fileWatcher.watch();
  }

  const farmWatcher = new ConfigWatcher({
    config: normalizedConfig,
    userConfig: config
  }).watch((filenames: string[]) => {
    logger.info(
      green(
        `${filenames
          .map((filename) => path.relative(config.root, filename))
          .join(', ')} changed, will restart server`
      )
    );

    farmWatcher.close();

    devServer.restart(async () => {
      fileWatcher?.close();
      await devServer.closeFarmServer();
      __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
      await start(inlineConfig);
    });
  });
}

export async function build(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('production');
  const userConfig: UserConfig = await resolveUserConfig(
    inlineConfig,
    'build',
    logger
  );
  const normalizedConfig = await normalizeUserCompilationConfig(
    inlineConfig,
    userConfig,
    logger,
    'production'
  );
  setProcessEnv(normalizedConfig.config.mode);

  await createBundleHandler(normalizedConfig, userConfig);

  // copy resources under publicDir to output.path
  const absPublicDirPath = normalizePublicDir(
    normalizedConfig.config.root,
    inlineConfig.publicDir
  );

  if (existsSync(absPublicDirPath)) {
    fse.copySync(absPublicDirPath, normalizedConfig.config.output.path);
  }
}

export async function preview(inlineConfig: FarmCLIOptions): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  const port = inlineConfig.port ?? 1911;
  const userConfig = await resolveUserConfig(inlineConfig, 'serve', logger);

  const normalizedConfig = await normalizeUserCompilationConfig(
    inlineConfig,
    userConfig,
    logger,
    'production'
  );
  const normalizedDevServerConfig = normalizeDevServerOptions(
    userConfig.server,
    'production'
  );

  const { root, output } = normalizedConfig.config;
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
  useProxy(normalizedDevServerConfig.proxy, app, logger);

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
    logger.info(green(`preview server running at:\n`));
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
          const url = `${'http'}://${host}:${bold(port)}${
            output.publicPath ?? ''
          }`;
          logger.info(`${magenta('>')} ${type} ${cyan(url)}`);
        })
    );
  });
}

export async function watch(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('development');
  const userConfig = await resolveUserConfig(inlineConfig, 'build', logger);
  const normalizedConfig = await normalizeUserCompilationConfig(
    inlineConfig,
    userConfig,
    logger,
    'development'
  );
  setProcessEnv(normalizedConfig.config.mode);

  const compilerFileWatcher = await createBundleHandler(
    normalizedConfig,
    userConfig,
    true
  );

  const farmWatcher = new ConfigWatcher({
    userConfig,
    config: normalizedConfig
  }).watch(async (files: string[]) => {
    logger.info(
      green(
        `${files
          .map((file) => path.relative(normalizedConfig.config.root, file))
          .join(', ')} changed, will be restart`
      )
    );

    farmWatcher.close();

    __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;

    compilerFileWatcher?.close();

    await watch(inlineConfig);
  });
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

  for (const nodeModulesPath of nodeModulesFolders) {
    const farmFolderPath = path.join(nodeModulesPath, '.farm');
    try {
      const stats = await fs.stat(farmFolderPath);
      if (stats.isDirectory()) {
        await fs.rm(farmFolderPath, { recursive: true, force: true });
        logger.info(
          `Under the current path, ${bold(
            green(nodeModulesPath)
          )}. The cache has been cleaned`
        );
      }
    } catch (error) {
      logger.warn(
        `Currently, no cached files have been found in ${bold(
          green(nodeModulesPath)
        )}.`
      );
    }
  }
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
  normalizedConfig: Config,
  userConfig: UserConfig,
  watchMode = false
) {
  const compiler = new Compiler(normalizedConfig);
  await compilerHandler(async () => {
    compiler.removeOutputPathDir();
    await compiler.compile();
    compiler.writeResourcesToDisk();

    // const maxFileNameLength = Math.max(
    //   ...Object.keys(compiler.resources()).map((name) => name.length)
    // );
    // const fileSizeMap = Object.entries(compiler.resources())
    //   .filter(([name]) => !name.endsWith('.map'))
    //   .map(([resourceName, resource]) => {
    //     let c = chalk.green;
    //     const size = Buffer.byteLength(resource) / 1024;

    //     if (size > 500) {
    //       c = chalk.yellow;
    //     }

    //     const sizeStr = c(size.toFixed(0)) + chalk.cyan(' KB');

    //     return {
    //       resourceName: resourceName.padEnd(maxFileNameLength + 4, ' '),
    //       size: sizeStr
    //     };
    //   });

    // console.log(`\n${chalk.green('Output Files:')}`);
    // fileSizeMap.forEach(({ resourceName, size }) =>
    //   console.log(`\t${chalk.cyan(resourceName)}\t${size}`)
    // );
  }, normalizedConfig);

  if (normalizedConfig.config?.watch || watchMode) {
    const watcher = new FileWatcher(compiler, {
      ...normalizedConfig,
      ...userConfig
    });

    await watcher.watch();

    return watcher;
  }
}

export { defineFarmConfig as defineConfig } from './config.js';
