import path from 'path';
import crypto from 'crypto';
import { createRequire } from 'module';
import {
  PreProcessors,
  PreProcessorsType,
  outputData,
  FarmVuePluginOptions,
  ResolvedOptions
} from './farm-vue-types.js';

export function warn({ id, message }: outputData) {
  console.warn(`[${id}:warn]:"${message}"`);
}

export function error({ id, message }: outputData) {
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

export function getHash(text: string, start = 0, end = 8) {
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

export function getResolvedOptions(defaultVueOptions: FarmVuePluginOptions) {
  const resolvedOptions: ResolvedOptions = {
    include: [],
    exclude: [],
    isProduction: false, // default: 'development'
    sourceMap: false,
    script: {},
    template: {},
    style: {}
  };
  for (const key in defaultVueOptions) {
    const val = defaultVueOptions[key as keyof FarmVuePluginOptions];
    switch (key) {
      case 'include':
        resolvedOptions.include = (
          isArray(val) ? val : [val]
        ) as ResolvedOptions['include'];
        break;
      case 'exclude':
        resolvedOptions.exclude = (
          isArray(val) ? val : [val]
        ) as ResolvedOptions['exclude'];
        break;
      case 'isProduction':
        if (val === true) resolvedOptions.isProduction = true;
        break;
      case 'sourceMap':
        if (val === true) resolvedOptions.sourceMap = true;
        break;
      case 'script':
        resolvedOptions.script = (val ? val : {}) as ResolvedOptions['script'];
        break;
      case 'template':
        resolvedOptions.template = (
          val ? val : {}
        ) as ResolvedOptions['template'];
        break;
      case 'style':
        resolvedOptions.style = (val ? val : {}) as ResolvedOptions['style'];
    }
  }
  resolvedOptions.sourceMap =
    resolvedOptions.isProduction === true ? false : true;
  return resolvedOptions;
}

export function handleInclude(resolvedOptions: ResolvedOptions) {
  return [
    ...new Set(
      resolvedOptions.include.map((match) => {
        return isRegExp(match) ? match.toString().slice(1, -1) : match;
      })
    )
  ];
}

export function handleExclude(resolvedOptions: ResolvedOptions) {
  return resolvedOptions.exclude.map((match) => {
    return isRegExp(match) ? match : new RegExp(match);
  });
}

export async function dynamicImportFromESM(moduleName: string) {
  const _require = createRequire(import.meta.url);
  const mod = _require(moduleName) ?? {};
  return mod.default ?? mod;
}

export async function loadPreProcessor<T extends PreProcessorsType>(
  lang: T
): Promise<PreProcessors[T]> {
  try {
    const preProcessor = await dynamicImportFromESM(lang);
    return preProcessor;
  } catch (error: any) {
    if (error.code === 'MODULE_NOT_FOUND') {
      throw new Error(
        `Preprocessor dependency "${lang}" not found. Did you install it?`
      );
    } else {
      const message = new Error(
        `Preprocessor dependency "${lang}" failed to load:\n${error.message}`
      );
      message.stack = error.stack + '\n' + message.stack;
      throw message;
    }
  }
}

export function isLess(
  preProcessor: unknown
): preProcessor is PreProcessors[PreProcessorsType.less] {
  return (
    typeof preProcessor !== 'function' &&
    'version' in (preProcessor as PreProcessors[PreProcessorsType.less])
  );
}

export function isSass(
  preProcessor: unknown
): preProcessor is PreProcessors[PreProcessorsType.sass] {
  return 'info' in (preProcessor as PreProcessors[PreProcessorsType.sass]);
}

export function isStyl(
  preProcessor: unknown
): preProcessor is PreProcessors[PreProcessorsType.stylus] {
  return (
    typeof preProcessor === 'function' &&
    'version' in (preProcessor as PreProcessors[PreProcessorsType.stylus])
  );
}
