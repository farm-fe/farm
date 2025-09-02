import fs from 'node:fs';
import { isAbsolute, join } from 'node:path';
import { Alias } from '../config/types.js';
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
  throw new Error(`[${pluginName} ${type} Error] ${error?.stack ?? error}`);
}

export function getAliasEntries(
  entries: Record<string, string> | Array<Alias>
): any {
  if (!entries || !Object.keys(entries).length) {
    return [];
  }

  if (Array.isArray(entries)) {
    return entries.map((entry) => {
      return {
        find: entry.find,
        replacement: entry.replacement
        // TODO add support for customResolver
      };
    });
  } else if (typeof entries === 'object') {
    return Object.entries(entries).map(([key, value]) => {
      return { find: key, replacement: value };
    });
  }

  // If entries is neither an array nor an object, return an empty array
  return [];
}

export function transformAliasWithVite(
  alias: Array<Alias>
): Record<string, string> {
  return alias.reduce<Record<string, string>>((acc, item) => {
    acc[item.find] = item.replacement;
    return acc;
  }, {});
}

export function removeSlash(path: string) {
  if (!path) return '';
  return path.replace(/^[/\\]+/, '');
}
