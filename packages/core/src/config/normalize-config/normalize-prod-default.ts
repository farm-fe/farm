import { Logger } from '../../utils/index.js';
import { ResolvedCompilation } from '../index.js';

export function normalizeProdDefaults(
  resolvedCompilation: ResolvedCompilation,
  isProduction: boolean,
  logger: Logger
) {
  if (resolvedCompilation.treeShaking === undefined) {
    resolvedCompilation.treeShaking ??= isProduction;
  }

  if (resolvedCompilation.concatenateModules === undefined) {
    resolvedCompilation.concatenateModules ??= isProduction;
  }

  if (resolvedCompilation.concatenateModules && !isProduction) {
    logger.warn(
      'concatenateModules option is not supported with development mode, concatenateModules will be disabled'
    );
    resolvedCompilation.concatenateModules = false;
  }

  if (resolvedCompilation.minify === undefined) {
    resolvedCompilation.minify ??= isProduction;
  }

  if (resolvedCompilation.presetEnv === undefined) {
    resolvedCompilation.presetEnv ??= isProduction;
  }

  if (resolvedCompilation.sourcemap === undefined) {
    // disable sourcemap in production mode
    resolvedCompilation.sourcemap ??= !isProduction;
  }
}
