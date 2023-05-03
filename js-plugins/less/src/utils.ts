import fs from 'fs';
import { createRequire } from 'module';
import { throwError } from './options.js';

const __require = createRequire(import.meta.url);

// export const logger:Logger = new DefaultLogger();

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
