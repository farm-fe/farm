import fs from 'node:fs';
import { isAbsolute, join } from 'node:path';
import { CompilationContext } from '../plugin/type.js';

export const getAdditionContext = async (
  cwd: string,
  option: {
    globals?: string[];
    additionalData?:
      | string
      | ((content: string, currentFile: string) => string | Promise<string>);
  },
  currentFile: string,
  content: string,
  ctx: CompilationContext,
  pluginName: string
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
      throwError(pluginName, 'read', error);
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

export function throwError(pluginName: string, type: string, error: Error) {
  throw new Error(`[${pluginName} ${type} Error] ${error}`);
}
