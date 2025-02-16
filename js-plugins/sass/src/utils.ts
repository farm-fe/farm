import { createRequire } from 'module';
import { throwError } from './options.js';

const __require = createRequire(import.meta.url);
/**
 * If not configured, the default implementation is to look for
 * @returns
 */
async function getDefaultSassImplementation() {
  let sassImplPkg = 'sass';
  try {
    __require.resolve('sass');
  } catch {
    try {
      __require.resolve('sass-embedded');
      sassImplPkg = 'sass-embedded';
    } catch {
      sassImplPkg = 'sass';
    }
  }
  return __require(sassImplPkg).default ?? __require(sassImplPkg);
}

/**
 * A function to getSassImplementation
 * @param implementation
 * @returns
 */
export function getSassImplementation(implementation?: string | any) {
  let resolvedImplementation = implementation;
  // if empty
  if (!implementation) {
    try {
      resolvedImplementation = getDefaultSassImplementation();
    } catch (error) {
      throwError('SassImplementation', error);
      process.exit(error);
    }
  }
  // if config
  if (typeof implementation === 'string') {
    try {
      resolvedImplementation = __require(implementation);
    } catch (error) {
      throwError('SassImplementation', error);
      process.exit(error);
    }
  }
  return resolvedImplementation;
}
