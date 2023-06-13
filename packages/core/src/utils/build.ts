import { performance } from 'node:perf_hooks';
import chalk from 'chalk';

import { DefaultLogger } from '../utils/logger.js';

import type { Config } from '../../binding/index.js';

export async function compilerHandler(
  callback: () => Promise<void>,
  config: Config
) {
  const logger = new DefaultLogger();
  const startTime = performance.now();
  try {
    await callback();
  } catch (error) {
    logger.error(error);
  }
  const endTime = performance.now();
  const elapsedTime = Math.floor(endTime - startTime);
  logger.info(
    `⚡️ Build completed in ${chalk.green(
      `${elapsedTime}ms`
    )}! Resources emitted to ${chalk.green(config.config.output.path)}.`
  );
}
