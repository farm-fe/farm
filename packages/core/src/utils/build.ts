import { performance } from 'node:perf_hooks';
import { DefaultLogger } from './logger.js';

import type { Config } from '../../binding/index.js';
import { bold, green } from './color.js';

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
    `⚡️ Build completed in ${green(
      `${elapsedTime}ms`
    )}! Resources emitted to ${bold(green(config.config.output.path))}.`
  );
}
