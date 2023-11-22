import { performance } from 'node:perf_hooks';
import { DefaultLogger } from './logger.js';

import type { Config } from '../../binding/index.js';
import { BrandText, bold, green } from './color.js';

export async function compilerHandler(
  callback: () => Promise<void>,
  config: Config
) {
  const usePersistentCache = config.config.persistentCache;
  const persistentCacheFlag = usePersistentCache ? bold(BrandText) : '';
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
    `⚡️ Build completed in ${bold(
      green(`${elapsedTime}ms`)
    )} ${persistentCacheFlag}! Resources emitted to ${bold(
      green(config.config.output.path)
    )}.`
  );
}
