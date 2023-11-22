export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';

import path from 'node:path';
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
  resolveConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import { Config } from '../binding/index.js';
import { compilerHandler } from './utils/build.js';
import { setProcessEnv } from './config/env.js';
import { JsPlugin } from './plugin/type.js';
import { bold, cyan, green, magenta } from './utils/color.js';
import { useProxy } from './server/middlewares/index.js';

import type { FarmCLIOptions } from './config/types.js';

export async function start(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();

  setProcessEnv('development');
  const { config, normalizedConfig } = await resolveConfig(
    inlineConfig,
    'serve',
    'development',
    logger
  );

  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, config, normalizedConfig);

  if (normalizedConfig.config.mode === 'development') {
    normalizedConfig.jsPlugins.forEach((plugin: JsPlugin) =>
      plugin.configDevServer?.(devServer)
    );
  }
  await devServer.listen();

  // Make sure the server is listening before we watch for file changes
  if (devServer.config.hmr) {
    if (normalizedConfig.config.mode === 'production') {
      logger.error(
        'HMR can not be enabled in production mode. Please set the mode option to "development" in your config file.'
      );
      process.exit(1);
    }
    const fileWatcher = new FileWatcher(devServer, {
      ...normalizedConfig,
      ...config
    });
    fileWatcher.watch();
  }
}

export async function build(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('production');
  
  const { normalizedConfig } = await resolveConfig(
    options,
    'serve',
    'production',
    logger
  );
  setProcessEnv(normalizedConfig.config.mode);

  await createBundleHandler(normalizedConfig);

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
  const userConfig: UserConfig = await resolveConfig(
    inlineConfig,
    'serve',
    'production',
    logger
  );

  const normalizedDevServerConfig = normalizeDevServerOptions(
    config.server,
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

  const { normalizedConfig } = await resolveConfig(
    inlineConfig,
    'serve',
    'development',
    logger
  );

  setProcessEnv(normalizedConfig.config.mode);

  createBundleHandler(normalizedConfig, true);
}

export async function createBundleHandler(
  normalizedConfig: Config,
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
    const watcher = new FileWatcher(compiler, normalizedConfig);
    watcher.watch();
  }
}

export { defineFarmConfig as defineConfig } from './config.js';
