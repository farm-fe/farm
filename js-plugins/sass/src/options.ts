import { isAbsolute, join } from 'path';
import fs from 'fs';
import { createRequire } from 'module';
import type { SassPluginOptions } from './index.js';

const __require = createRequire(__dirname);

export const { name: pluginName } = __require('../package.json');

export const getAdditionContext = (cwd: string, option: SassPluginOptions) => {
  const { globals = [], content } = option;

  const result = globals.reduce((result, file) => {
    let filepath: string;
    if (isAbsolute(file)) {
      filepath = file;
    } else {
      filepath = join(cwd, file);
    }
    try {
      result.push(fs.readFileSync(filepath, 'utf-8'));
    } catch (error) {
      throwError('read', error);
    }
    return result;
  }, []);
  if (content) {
    result.push(content);
  }

  return result.join('\n');
};

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}

export async function tryRead(filename: string) {
  try {
    return await fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('read', e);
  }
}
