
import { pluginName } from './options.js';
import { createRequire } from 'module';
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
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  return __require(sassImplPkg).default??__require(sassImplPkg);
}

/**
 * A function to getSassImplementation
 * @param implementation 
 * @returns 
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getSassImplementation(implementation?:any) {
  let resolvedImplementation = implementation;
  // if empty
  if(!resolvedImplementation) {
    try{
      resolvedImplementation = getDefaultSassImplementation();
    }catch(error) {
      throwError("Unknown",error);
      process.exit(error);
    }
  }
  // if conifg
  if(typeof implementation === "string") {
    try{
      resolvedImplementation = __require(implementation);
    }catch(error) {
      throwError("Unknown",error);
      process.exit(error);
    }
  }
  return resolvedImplementation;
}

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}
