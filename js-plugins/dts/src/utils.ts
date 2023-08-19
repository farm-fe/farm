import path, {
  posix,
  isAbsolute,
  dirname,
  normalize,
  sep,
  resolve
} from 'node:path';
import { existsSync, lstatSync, readdirSync, rmdirSync } from 'node:fs';

import typescript from 'typescript';
import extra from 'fs-extra';

import crypto from 'crypto';
import fs from 'node:fs';
import { createRequire } from 'module';
import { CompilerOptions } from 'ts-morph';
import { throwError } from './options.js';

const windowsSlashRE = /\\+/g;
const globSuffixRE = /^((?:.*\.[^.]+)|(?:\*+))$/;

// @ts-ignore
export function warn({ id, message }) {
  console.warn(`[${id}:warn]:"${message}"`);
}

// @ts-ignore
export function error({ id, message }) {
  console.error(`[${id}-(error)]:"${message}"`);
}

export function parsePath(resolvedPath: string) {
  const { dir, base } = path.parse(resolvedPath);
  const [filename, query] = base.split('?');
  const queryObj: Record<string, string> = {};
  if (query) {
    query.split('&').forEach((keyValue) => {
      const [key, value] = keyValue.split('=');
      queryObj[key] = value;
    });
  }
  return {
    filename,
    filePath: path.join(dir, filename),
    query: queryObj
  };
}

export function slash(p: string): string {
  return p.replace(windowsSlashRE, '/');
}

export function normalizePath(id: string): string {
  return posix.normalize(slash(id));
}

export function getHash(text: string, start: number = 0, end: number = 8) {
  return crypto
    .createHash('sha256')
    .update(text)
    .digest('hex')
    .substring(start, end)
    .toLocaleLowerCase();
}

export function callWithErrorHandle<
  T,
  U extends (...args: unknown[]) => unknown,
  M extends unknown[]
>(_this: T, fn: U, args: M) {
  try {
    const result = fn.call(_this, ...args) as ReturnType<U>;
    return result;
  } catch (e) {
    console.error(e);
  }
}

export function isNativeObj<
  T extends Record<string, any> = Record<string, any>
>(value: T): value is T {
  return Object.prototype.toString.call(value) === '[object Object]';
}

export function isPromise(value: unknown): value is Promise<any> {
  return (
    !!value &&
    (typeof value === 'function' || typeof value === 'object') &&
    typeof (value as any).then === 'function'
  );
}

export function isArray(val: unknown): val is unknown[] {
  return Array.isArray(val);
}

export function isRegExp(reg: unknown): reg is RegExp {
  return Object.prototype.toString.call(reg) === '[object RegExp]';
}

export function getResolvedOptions(defaultVueOptions: any) {
  const resolvedOptions: any = {
    include: [],
    exclude: [],
    isProduction: false, // default: 'development'
    sourceMap: false
  };
  for (const key in defaultVueOptions) {
    const val = defaultVueOptions[key as keyof any];
    switch (key) {
      case 'include':
        resolvedOptions.include = (
          isArray(val) ? val : [val]
        ) as any['include'];
      case 'exclude':
        resolvedOptions.exclude = (
          isArray(val) ? val : [val]
        ) as any['exclude'];
      case 'isProduction':
        if (val === true) resolvedOptions.isProduction = true;
      case 'sourceMap':
        if (val === true) resolvedOptions.sourceMap = true;
      case 'script':
        resolvedOptions.script = (val ? val : {}) as any['script'];
      case 'template':
        resolvedOptions.template = (val ? val : {}) as any['template'];
      case 'style':
        resolvedOptions.style = (val ? val : {}) as any['style'];
    }
  }
  resolvedOptions.sourceMap =
    resolvedOptions.isProduction === true ? false : true;
  return resolvedOptions;
}

export function handleInclude(resolvedOptions: any) {
  return [
    ...new Set(
      resolvedOptions.include.map((match: any) => {
        return isRegExp(match) ? match.toString().slice(1, -1) : match;
      })
    )
  ];
}

export function handleExclude(resolvedOptions: any) {
  return resolvedOptions.exclude.map((match: any) => {
    return isRegExp(match) ? match : new RegExp(match);
  });
}

export async function dynamicImportFromESM(moduleName: string) {
  try {
    // @ts-ignore
    // TODO: use dynamic import
    const _require = createRequire(import.meta.url);
    const module = _require(moduleName) ?? {};
    return module.default ?? module;
  } catch (error) {
    throw error;
  }
}

export function resolveAbsolutePath(path: string, root: string) {
  return path ? (isAbsolute(path) ? path : resolve(root, path)) : root;
}

export function mergeObjects<
  T extends Record<string, unknown>,
  U extends Record<string, unknown>
>(sourceObj: T, targetObj: U) {
  const loop: Array<{
    source: Record<string, any>;
    target: Record<string, any>;
    // merged: Record<string, any>
  }> = [
    {
      source: sourceObj,
      target: targetObj
      // merged: mergedObj
    }
  ];

  while (loop.length) {
    const { source, target } = loop.pop()!;

    Object.keys(target).forEach((key) => {
      if (isObject(target[key])) {
        if (!isObject(source[key])) {
          source[key] = {};
        }

        loop.push({
          source: source[key],
          target: target[key]
        });
      } else if (Array.isArray(target[key])) {
        if (!Array.isArray(source[key])) {
          source[key] = [];
        }

        loop.push({
          source: source[key],
          target: target[key]
        });
      } else {
        source[key] = target[key];
      }
    });
  }

  return sourceObj as T & U;
}

export function isObject<T extends Record<string, any> = Record<string, any>>(
  value: T
): value is T {
  return Object.prototype.toString.call(value) === '[object Object]';
}

export function getTsConfig(
  tsConfigPath: string,
  readFileSync: (filePath: string, encoding?: string | undefined) => string
) {
  // #95 Should parse include or exclude from the base config when they are missing from
  // the inheriting config. If the inherit config doesn't have `include` or `exclude` field,
  // should get them from the parent config.
  const tsConfig: {
    compilerOptions: CompilerOptions;
    include?: string[];
    exclude?: string[];
    extends?: string | string[];
  } = {
    compilerOptions: {},
    ...(typescript.readConfigFile(tsConfigPath, readFileSync).config ?? {})
  };

  if (tsConfig.extends) {
    ensureArray(tsConfig.extends).forEach((configPath: string) => {
      const config = getTsConfig(
        resolveAbsolutePath(configPath, dirname(tsConfigPath)),
        readFileSync
      );

      // #171 Need to collect the full `compilerOptions` for `@microsoft/api-extractor`
      Object.assign(tsConfig.compilerOptions, config.compilerOptions);
      if (!tsConfig.include) {
        tsConfig.include = config.include;
      }

      if (!tsConfig.exclude) {
        tsConfig.exclude = config.exclude;
      }
    });
  }

  return tsConfig;
}

export function ensureArray<T>(value: T | T[]) {
  return Array.isArray(value) ? value : value ? [value] : [];
}

export async function tryRead(filename: string) {
  try {
    return await fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('readFile', e);
  }
}

export function ensureAbsolute(path: string, root: string) {
  return path ? (isAbsolute(path) ? path : resolve(root, path)) : root;
}

export function queryPublicPath(paths: string[]) {
  const speRE = /[\\/]/;

  if (paths.length === 0) {
    return '';
  } else if (paths.length === 1) {
    return dirname(paths[0]);
  }

  let publicPath = normalize(dirname(paths[0])) + sep;
  let publicUnits = publicPath.split(speRE);
  let index = publicUnits.length - 1;

  for (const path of paths.slice(1)) {
    if (!index) {
      return publicPath;
    }

    const dirPath = normalize(dirname(path)) + sep;

    if (dirPath.startsWith(publicPath)) {
      continue;
    }

    const units = dirPath.split(speRE);

    if (units.length < index) {
      publicPath = dirPath;
      publicUnits = units;
      continue;
    }

    for (let i = 0; i <= index; ++i) {
      if (publicUnits[i] !== units[i]) {
        if (!i) {
          return '';
        }

        index = i - 1;
        publicUnits = publicUnits.slice(0, index + 1);
        publicPath = publicUnits.join(sep) + sep;
        break;
      }
    }
  }

  return publicPath.slice(0, -1);
}

export async function runParallel<T>(
  maxConcurrency: number,
  source: T[],
  iteratorFn: (item: T, source: T[]) => Promise<any>
) {
  const ret: Promise<any>[] = [];
  const executing: Promise<any>[] = [];

  for (const item of source) {
    const p = Promise.resolve().then(() => iteratorFn(item, source));

    ret.push(p);

    if (maxConcurrency <= source.length) {
      const e: Promise<any> = p.then(() =>
        executing.splice(executing.indexOf(e), 1)
      );

      executing.push(e);

      if (executing.length >= maxConcurrency) {
        await Promise.race(executing);
      }
    }
  }

  return Promise.all(ret);
}

export async function tryToReadFileSync(path: string) {
  try {
    return await fs.promises.readFile(path, 'utf-8');
  } catch (error) {
    console.error(`[Farm Plugin Dts]: ${error.type}: ${error.message}`);
  }
}

export function normalizeGlob(path: string) {
  if (/[\\/]$/.test(path)) {
    return path + '**';
  } else if (!globSuffixRE.test(path.split(/[\\/]/).pop()!)) {
    return path + '/**';
  }

  return path;
}

export async function writeFileWithCheck(filePath: string, content: string) {
  // 获取文件夹路径
  const folderPath = path.dirname(filePath);
  try {
    // 检查文件夹是否存在
    await extra.access(folderPath);
  } catch (error) {
    // 如果文件夹不存在，则创建它
    await extra.mkdir(folderPath, { recursive: true });
  }

  // 写文件
  await extra.writeFile(filePath, content, 'utf-8');
}
