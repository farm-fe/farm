import { performance } from 'node:perf_hooks';
import { DefaultLogger } from './logger.js';

import type { Config } from '../../binding/index.js';
import { PersistentCacheBrand, bold, green } from './color.js';

export async function compilerHandler(
  callback: () => Promise<void>,
  config: Config
) {
  const usePersistentCache = config.config.persistentCache;
  const persistentCacheText = usePersistentCache
    ? bold(PersistentCacheBrand)
    : '';
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
    `Build completed in ${bold(
      green(`${elapsedTime}ms`)
    )} ${persistentCacheText} Resources emitted to ${bold(
      green(config.config.output.path)
    )}.`
  );
}
