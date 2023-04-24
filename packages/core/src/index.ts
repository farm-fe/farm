export * from "./compiler/index.js";
export * from "./config/index.js";
export * from "./server/index.js";
export * from "./plugin/index.js";

import chalk from "chalk";
import { Compiler } from "./compiler/index.js";
import {
  normalizeUserCompilationConfig,
  resolveUserConfig,
  UserConfig,
} from "./config/index.js";
import { DefaultLogger, Logger } from "./logger.js";
import { DevServer } from "./server/index.js";
import { FileWatcher } from "./watcher/index.js";

export async function start(options: {
  configPath?: string;
  logger?: Logger;
}): Promise<void> {
  console.log(options);

  const logger = options.logger ?? new DefaultLogger();
  const userConfig: UserConfig = await resolveUserConfig(
    options,
    logger
  );
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    "development"
  );
  const compiler = new Compiler(normalizedConfig);
  const devServer = new DevServer(compiler, logger, userConfig.server);

  await devServer.listen();
  // Make sure the server is listening before we watch for file changes
  if (devServer.config.hmr) {
    logger.info(
      "HMR enabled, watching for file changes under " +
        chalk.green(userConfig.root)
    );

    if (normalizedConfig.config.mode === "production") {
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
    options.configPath,
    logger
  );
  const normalizedConfig = await normalizeUserCompilationConfig(
    userConfig,
    "production"
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
