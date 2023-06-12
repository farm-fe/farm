export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/index.js';
export * from './utils/index.js';

import path from 'node:path';
import os from 'node:os';
import { existsSync } from 'node:fs';

import fse from 'fs-extra';
import chalk from 'chalk';
import sirv from 'sirv';
import compression from 'koa-compress';
import Koa, { Context } from 'koa';
import { Compiler } from './compiler/index.js';
import {
  normalizePublicDir,
  normalizeUserCompilationConfig,
  resolveInlineConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import { Config } from '../binding/index.js';
import { compilerHandler } from './utils/build.js';

import type { FarmCLIOptions } from './config/types.js';

export async function start(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  const config: UserConfig = await resolveInlineConfig(inlineConfig, logger);
  const normalizedConfig = await normalizeUserCompilationConfig(config);
  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, config.server);
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

    const fileWatcher = new FileWatcher(devServer, normalizedConfig);
    fileWatcher.watch();
  }
}

export async function build(
  options: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();
  const userConfig: UserConfig = await resolveInlineConfig(options, logger);
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'production'
  );

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
  const userConfig: UserConfig = await resolveInlineConfig(options, logger);

  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
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
          const url = `${'http'}://${host}:${chalk.bold(port)}`;
          logger.info(`  > ${type} ${chalk.cyan(url)}`);
        })
    );
  });
}

export async function watch(
  options: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();
  const userConfig: UserConfig = await resolveInlineConfig(options, logger);
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'production'
  );

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
  }, normalizedConfig);

  if (normalizedConfig.config?.watch || watchMode) {
    const watcher = new FileWatcher(compiler, normalizedConfig);
    watcher.watch();
  }
}
