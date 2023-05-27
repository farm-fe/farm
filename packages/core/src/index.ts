export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/index.js';
export * from './utils/index.js';

import path from 'node:path';
import os from 'node:os';
import chalk from 'chalk';
import sirv from 'sirv';
import compression from 'koa-compress';
import Koa, { Context } from 'koa';
import { Compiler } from './compiler/index.js';
import {
  normalizeUserCompilationConfig,
  resolveInlineConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import type { FarmCLIOptions } from './config/types.js';
import { Config } from '../binding/index.js';

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

    const fileWatcher = new FileWatcher(config.root, devServer.config.hmr);
    fileWatcher.watch(devServer, {});
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

  const start = Date.now();
  const compiler = new Compiler(normalizedConfig);
  compiler.removeOutputPathDir();
  if (userConfig.compilation.watch) {
    createFileWatcher(userConfig.root, compiler, normalizedConfig);
  } else {
    await compiler.compile();
    compiler.writeResourcesToDisk();
    logger.info(
      `⚡️ Build completed in ${chalk.green(
        `${Date.now() - start}ms`
      )}! Resources emitted to ${chalk.green(
        normalizedConfig.config.output.path
      )}.`
    );
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
          logger.info(`  > ${type} ${chalk.cyan(url)}`, false);
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
  const compiler = new Compiler(normalizedConfig);
  createFileWatcher(userConfig.root, compiler, normalizedConfig);
}

export function createFileWatcher(
  watcherDirPath: string,
  compiler: Compiler,
  normalizedConfig: Config
) {
  const outDir = normalizedConfig.config.output.path;
  const outDirRegex = new RegExp(`^.*${outDir}.*$`);
  const fileWatcher = new FileWatcher(watcherDirPath, {
    ignores: ['**/{.git,node_modules}/**', outDirRegex]
  });
  fileWatcher.watch(compiler, normalizedConfig);
}
