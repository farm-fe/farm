export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';

import path from 'node:path';
import os from 'node:os';
import { existsSync } from 'node:fs';
import chalk from 'chalk';
import sirv from 'sirv';
import compression from 'koa-compress';
import Koa, { Context } from 'koa';
import fse from 'fs-extra';
import { Compiler } from './compiler/index.js';
import {
  normalizePublicDir,
  normalizeUserCompilationConfig,
  resolveUserConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger, Logger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import { Config } from '../binding/index.js';
import { compilerHandler } from './utils/build.js';

import type { FarmCLIOptions } from './config/types.js';
import { setProcessEnv } from './config/env.js';
import { JsPlugin } from './plugin/type.js';

export async function start(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();

  setProcessEnv('development');
  const config: UserConfig = await resolveUserConfig(inlineConfig, logger);
  const normalizedConfig = await normalizeUserCompilationConfig(config, logger);

  setProcessEnv(normalizedConfig.config.mode);

  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, config);

  if (normalizedConfig.config.mode === 'development') {
    normalizedConfig.jsPlugins.forEach((plugin: JsPlugin) =>
      plugin.configDevServer?.(devServer)
    );
  }
  await devServer.listen();
  // Make sure the server is listening before we watch for file changes
  if (devServer.config.hmr) {
    logger.info(
      'HMR enabled, watching for file changes under ' + chalk.green(config.root)
    );

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
  options: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();
  setProcessEnv('production');
  const userConfig: UserConfig = await resolveUserConfig(options, logger);
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    logger,
    'production'
  );
  setProcessEnv(normalizedConfig.config.mode);

  await createBundleHandler(normalizedConfig);

  // copy resources under publicDir to output.path
  const absPublicDirPath = normalizePublicDir(
    normalizedConfig.config.root,
    options.publicDir
  );

  if (existsSync(absPublicDirPath)) {
    fse.copySync(absPublicDirPath, normalizedConfig.config.output.path);
  }
}

export async function preview(options: FarmCLIOptions): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();
  const port = options.port ?? 1911;
  const userConfig: UserConfig = await resolveUserConfig(options, logger);

  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    logger,
    'production'
  );
  const { root, output } = normalizedConfig.config;
  const distDir = path.resolve(root, output.path);

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
    logger.info(chalk.green(`preview server running at:\n`));
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
          const url = `${'http'}://${host}:${chalk.bold(port)}${
            output.publicPath ?? ''
          }`;
          logger.info(`${chalk.magenta('>')} ${type} ${chalk.cyan(url)}`);
        })
    );
  });
}

export async function watch(
  options: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();
  setProcessEnv('development');
  const userConfig: UserConfig = await resolveUserConfig(options, logger);
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    logger,
    'development'
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

export async function resolveCompiler(
  inlineConfig: FarmCLIOptions & UserConfig & Config,
  logger: Logger
) {
  setProcessEnv('development');
  const config: UserConfig = await resolveUserConfig(inlineConfig, logger);
  const normalizedConfig = await normalizeUserCompilationConfig(config, logger);

  setProcessEnv(normalizedConfig.config.mode);

  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, config);

  if (normalizedConfig.config.mode === 'development') {
    normalizedConfig.jsPlugins.forEach((plugin: JsPlugin) =>
      plugin.configDevServer?.(devServer)
    );
  }
  await devServer.listen();
  return {
    devServer,
    normalizedConfig,
    config
  };
}
