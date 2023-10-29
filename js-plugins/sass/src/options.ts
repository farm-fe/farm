import { isAbsolute, join } from 'path';
import fs from 'fs';
import { createRequire } from 'module';
import type { SassPluginOptions } from './index.js';
import { CompilationContext } from '@farmfe/core';

const __require = createRequire(__filename);

export const { name: pluginName } = __require('../package.json');

export const getAdditionContext = async (
  cwd: string,
  option: SassPluginOptions,
  currentFile: string,
  content: string,
  ctx: CompilationContext
) => {
  const { globals = [], additionalData } = option;

  const result = globals.reduce((result, file) => {
    let filepath: string;
    if (isAbsolute(file)) {
      filepath = file;
    } else {
      filepath = join(cwd, file);
    }
    try {
      result.push(fs.readFileSync(filepath, 'utf-8'));

      ctx.addWatchFile(currentFile, filepath);
    } catch (error) {
      throwError('read', error);
    }
    return result;
  }, []);

  if (additionalData) {
    if (typeof additionalData === 'string') {
      result.push(additionalData);
    } else {
      result.push(await additionalData(content, currentFile));
    }
  }

  return result.join('\n');
};

export function throwError(type: string, error: Error) {
  throw new Error(`[${pluginName} ${type} Error] ${error}`);
}

export async function tryRead(filename: string) {
  try {
    return await fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('read', e);
  }
}
