import path from 'path';
import crypto from 'crypto';
import { createRequire } from 'module';

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
