import fs from 'fs';
import { createRequire } from 'module';
import pkg from '../package.json';

export const pluginName = pkg.name;

const __require = createRequire(__filename);

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
