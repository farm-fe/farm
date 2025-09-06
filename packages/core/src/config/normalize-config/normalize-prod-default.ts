import type { Logger } from '../../utils/index.js';
import type { ResolvedCompilation } from '../index.js';

export function normalizeProdDefaults(
  resolvedCompilation: ResolvedCompilation,
  isProduction: boolean,
  logger: Logger
) {
  resolvedCompilation.treeShaking ??= isProduction;

  resolvedCompilation.concatenateModules ??= isProduction;

  if (resolvedCompilation.concatenateModules && !isProduction) {
    logger.warn(
      'concatenateModules option is not supported with development mode, concatenateModules will be disabled'
    );
    resolvedCompilation.concatenateModules = false;
  }

  resolvedCompilation.minify ??= isProduction;

  resolvedCompilation.presetEnv ??= isProduction;

  // disable sourcemap in production mode
  resolvedCompilation.sourcemap ??= !isProduction;
}
