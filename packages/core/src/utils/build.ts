import { performance } from 'node:perf_hooks';
import { DefaultLogger } from './logger.js';

import { PersistentCacheBrand, bold, green } from './color.js';
import { ResolvedUserConfig } from '../index.js';

export async function compilerHandler(
  callback: () => Promise<void>,
  config: ResolvedUserConfig
) {
  const { persistentCache, output } = config.compilation;
  const logger = new DefaultLogger();
  const startTime = performance.now();

  try {
    await callback();
  } catch (error) {
    logger.error(error, { exit: true });
    return;
  }

  const elapsedTime = Math.floor(performance.now() - startTime);
  const persistentCacheText = persistentCache ? bold(PersistentCacheBrand) : '';
  logger.info(
    `Build completed in ${bold(
      green(`${elapsedTime}ms`)
    )} ${persistentCacheText} Resources emitted to ${bold(green(output.path))}.`
  );
}
