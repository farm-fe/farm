import { performance } from 'node:perf_hooks';
import { DefaultLogger } from './logger.js';

import { PersistentCacheBrand, bold, green } from './color.js';
import { ResolvedUserConfig } from '../index.js';

export async function compilerHandler(
  callback: () => Promise<void>,
  config: ResolvedUserConfig
) {
  const usePersistentCache = config.compilation.persistentCache;
  const persistentCacheText = usePersistentCache
    ? bold(PersistentCacheBrand)
    : '';
  const logger = new DefaultLogger();
  const startTime = performance.now();
  try {
    await callback();
  } catch (error) {
    console.log('这个应该是构建的报错');
    logger.error(error, {
      exit: true
    });
  }
  const endTime = performance.now();
  const elapsedTime = Math.floor(endTime - startTime);
  logger.info(
    `Build completed in ${bold(
      green(`${elapsedTime}ms`)
    )} ${persistentCacheText} Resources emitted to ${bold(
      green(config.compilation.output.path)
    )}.`
  );
}
