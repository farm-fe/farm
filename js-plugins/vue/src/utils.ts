import path from 'path';
import crypto from 'crypto';
import {
  FarmVuePluginOptions,
  outputData,
  ResolvedOptions,
} from './farm-vue-types';
function warn({ id, message }: outputData) {
  console.warn(`[${id}:warn]:"${message}"`);
}

function error({ id, message }: outputData) {
  console.error(`[${id}-(error)]:"${message}"`);
}

function parsePath(resolvedPath: string) {
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
    query: queryObj,
  };
}

function getHash(text: string, start: number = 0, end: number = 8) {
  return crypto
    .createHash('sha256')
    .update(text)
    .digest('hex')
    .substring(start, end)
    .toLocaleLowerCase();
}

function callWithErrorHandle<
  T,
  U extends (...args: any[]) => any,
  M extends any[]
>(_this: T, fn: U, args: M) {
  try {
    const result = fn.call(_this, ...args) as ReturnType<U>;
    return result;
  } catch (e) {
    console.error(e);
  }
}

function isArray(val: any) {
  return Array.isArray(val);
}

function getResolvedOptions(defaultVueOptions: FarmVuePluginOptions) {
  const resolvedOptions: ResolvedOptions = {
    include: [],
    exclude: [],
    isProduction: false, // default: 'development'
    sourceMap: false,
    script: {},
    template: {},
    style: {},
  };
  for (const key in defaultVueOptions) {
    const val = defaultVueOptions[key as keyof FarmVuePluginOptions];
    switch (key) {
      case 'include':
        resolvedOptions.include = (
          isArray(val) ? val : [val]
        ) as ResolvedOptions['include'];
      case 'exclude':
        resolvedOptions.exclude = (
          isArray(val) ? val : [val]
        ) as ResolvedOptions['exclude'];
      case 'isProduction':
        if (val === true) resolvedOptions.isProduction = true;
      case 'sourceMap':
        if (val === true) resolvedOptions.sourceMap = true;
      case 'script':
        resolvedOptions.script = (val ? val : {}) as ResolvedOptions['script'];
      case 'template':
        resolvedOptions.template = (
          val ? val : {}
        ) as ResolvedOptions['template'];
      case 'style':
        resolvedOptions.style = (val ? val : {}) as ResolvedOptions['style'];
    }
  }
  resolvedOptions.sourceMap =
    resolvedOptions.isProduction === true ? false : true;
  return resolvedOptions;
}

function isRegExp(reg: any) {
  return Object.prototype.toString.call(reg) === '[object RegExp]';
}

function handleInclude(resolvedOptions: ResolvedOptions) {
  return [
    ...new Set(
      resolvedOptions.include.map((match) => {
        return isRegExp(match)
          ? match.toString().slice(1, -1)
          : (match as string);
      })
    ),
  ];
}

function handleExclude(resolvedOptions: ResolvedOptions) {
  return resolvedOptions.exclude.map((match) => {
    return isRegExp(match) ? (match as RegExp) : new RegExp(match);
  });
}

export {
  isArray,
  getResolvedOptions,
  callWithErrorHandle,
  getHash,
  parsePath,
  error,
  warn,
  handleExclude,
  handleInclude,
  isRegExp,
};
