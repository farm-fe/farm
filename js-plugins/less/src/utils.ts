import fs from 'fs';
import { createRequire } from 'module';

const __require = createRequire(__filename);

export const { name: pluginName } = __require('../package.json');

export function getLessImplementation(implementation?: string) {
  let resolvedImplementation;
  if (!implementation || typeof implementation === 'string') {
    const lessImplPkg = implementation || 'less';
    try {
      resolvedImplementation = __require(lessImplPkg);
    } catch (e) {
      throwError('Implementation', e);
    }
  }
  return resolvedImplementation;
}

export async function tryRead(filename: string) {
  try {
    return await fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('readFile', e);
  }
}

export function formatError(type: string, error: Error) {
  return `[${pluginName} ${type} Error] ${error}`;
}

export function throwError(type: string, error: Error) {
  throw new Error(formatError(type, error));
}
