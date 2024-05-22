import { performance } from 'node:perf_hooks';
import { Logger } from './logger.js';

import {
  FARM_TARGET_NODE_ENVS,
  ResolvedUserConfig,
  clearScreen
} from '../index.js';
import { logError } from '../server/error.js';
import { PersistentCacheBrand, bold, green } from './color.js';

interface CompilerHandlerOptions {
  clear?: boolean;
}

export async function compilerHandler(
  callback: () => Promise<void>,
  config: ResolvedUserConfig,
  logger: Logger,
  options?: CompilerHandlerOptions
) {
  const IS_TARGET_NODE = FARM_TARGET_NODE_ENVS.includes(
    config.compilation.output.targetEnv
  );
  IS_TARGET_NODE && options?.clear && clearScreen();
  const { persistentCache, output } = config.compilation;
  const startTime = performance.now();

  try {
    await callback();
  } catch (error) {
    logger.error(`Compiler ${logError(error, false)}`);
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
