import fs from 'fs';
import { createRequire } from 'module';

const __require = createRequire(import.meta.url);

export const { name: pluginName } = __require('../../package.json');

export function getPostcssImplementation(implementation?: string) {
  let resolvedImplementation;
  if (!implementation || typeof implementation === 'string') {
    const lessImplPkg = implementation || 'postcss';
    try {
      resolvedImplementation = __require(lessImplPkg);
    } catch (e) {
      throwError('Implementation', e);
    }
  }
  return resolvedImplementation;
}

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}

export async function tryRead(filename: string) {
  try {
    return await fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('readFile', e);
  }
}
