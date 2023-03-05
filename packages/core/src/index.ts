export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';

import chalk from 'chalk';
import { Compiler } from './compiler/index.js';
import {
  normalizeUserCompilationConfig,
  resolveUserConfig,
  UserConfig,
} from './config/index.js';
import { DefaultLogger, Logger } from './logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';

export async function start(options: {
  configPath?: string;
  logger?: Logger;
}): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();
  const userConfig: UserConfig = await resolveUserConfig(
    options.configPath,
    logger
  );
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'development'
  );
  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, userConfig.server);

  if (devServer.config.hmr) {
    logger.info(
      'HMR enabled, watching for file changes under ' +
        chalk.green(userConfig.root)
    );
    const fileWatcher = new FileWatcher(userConfig.root, devServer.config.hmr);
    fileWatcher.watch(devServer);
  }

  devServer.listen();
}

export async function build(options: {
  configPath?: string;
  logger?: Logger;
}): Promise<void> {
  const logger = options.logger ?? new DefaultLogger();
  const userConfig: UserConfig = await resolveUserConfig(
    options.configPath,
    logger
  );
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    'production'
  );
  const start = Date.now();
  const compiler = new Compiler(normalizedConfig);
  await compiler.compile();
  logger.info(`Build completed in ${chalk.green(`${Date.now() - start}ms`)}!`);
}
