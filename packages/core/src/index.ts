export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/index.js';

// import http from 'http';
import chalk from 'chalk';
// import fs from 'fs';
// import chokidar from 'chokidar';
import sirv from 'sirv';
import os from 'node:os';
import compression from 'koa-compress';
import Koa, { Context } from 'koa';
import { Compiler } from './compiler/index.js';
import {
  normalizeUserCompilationConfig,
  resolveUserConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger, Logger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import type { FarmCLIOptions } from './config/types.js';
import path from 'path';

export async function start(options: FarmCLIOptions): Promise<void> {
  // TODO merger config options Encapsulation universal

  const logger = options.logger ?? new DefaultLogger();
  const userConfig: UserConfig = await resolveUserConfig(
    options,
    logger,
    'start'
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'development'
  );
  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, userConfig.server);

  await devServer.listen();
  // Make sure the server is listening before we watch for file changes
  if (devServer.config.hmr) {
    logger.info(
      'HMR enabled, watching for file changes under ' +
        chalk.green(userConfig.root)
    );

    if (normalizedConfig.config.mode === 'production') {
      logger.error(
        'HMR can not be enabled in production mode. Please set the mode option to "development" in your config file.'
      );
      process.exit(1);
    }

    const fileWatcher = new FileWatcher(userConfig.root, devServer.config.hmr);
    fileWatcher.watch(devServer);
  }
}

export async function build(options: {
  configPath?: string;
  logger?: Logger;
}): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();

  const userConfig: UserConfig = await resolveUserConfig(
    options,
    logger,
    'build'
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'production'
  );

  const start = Date.now();
  const compiler = new Compiler(normalizedConfig);
  compiler.removeOutputPathDir();
  await compiler.compile();
  compiler.writeResourcesToDisk();
  logger.info(
    `Build completed in ${chalk.green(
      `${Date.now() - start}ms`
    )}! Resources emitted to ${chalk.green(
      normalizedConfig.config.output.path
    )}.`
  );
}

export async function preview(
  options: FarmCLIOptions,
  port = 1911
): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();

  const userConfig: UserConfig = await resolveUserConfig(
    options,
    logger,
    'start'
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'production'
  );

  const { root, output } = normalizedConfig.config;
  const distDir = path.resolve(root, output.path);

  const app = new Koa();

  function StaticFilesHandler(ctx: Context) {
    const staticFilesHandler = sirv(distDir, {
      etag: true,
      single: true
    });
    return new Promise<void>((resolve) => {
      staticFilesHandler(ctx.req, ctx.res, () => {
        resolve();
      });
    });
  }

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

export async function watch(options: {
  configPath?: string;
  logger?: Logger;
  watchPath?: string;
}): Promise<void> {
  const watcherPath = options.watchPath;
  options.configPath = watcherPath;
  const logger = options.logger ?? new DefaultLogger();
  const userConfig: UserConfig = await resolveUserConfig(
    options,
    logger,
    'build'
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'production'
  );
  const compiler = new Compiler(normalizedConfig);
  const fileWatcher = new FileWatcher(watcherPath, {
    ignores: [/node_modules/, /dist/]
  });
  fileWatcher.watch(compiler);
  // const watcher = chokidar.watch(watcherPath, {});
  // build(options);

  // // 监听文件变化事件
  // watcher.on('change', (path) => {
  //   // 读取文件内容
  //   fs.readFile(path, 'utf8', async (err) => {
  //     if (err) {
  //       console.error(err, '编译报错了');
  //     } else {
  //       // const start = Date.now();
  //       // compiler.removeOutputPathDir();
  //       // console.log(path);

  //       // await compiler.update([path]);
  //       // compiler.writeResourcesToDisk();
  //       // logger.info(
  //       //   `Build completed in ${chalk.green(
  //       //     `${Date.now() - start}ms`
  //       //   )}! Resources emitted to ${chalk.green(
  //       //     normalizedConfig.config.output.path
  //       //   )}.`
  //       // );

  //       build({
  //         configPath: watcherPath
  //       });
  //     }
  //   });
  // });
}
