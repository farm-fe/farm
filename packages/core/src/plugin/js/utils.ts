// import path from 'node:path';
import * as querystring from 'node:querystring';
import { addSlashes, removeSlashes } from 'slashes';

export type WatchChangeEvents = 'create' | 'update' | 'delete';

// const ExtToLoader: Record<string, string> = {
//   '.js': 'js',
//   '.mjs': 'js',
//   '.cjs': 'js',
//   '.jsx': 'jsx',
//   '.ts': 'ts',
//   '.cts': 'ts',
//   '.mts': 'ts',
//   '.tsx': 'tsx',
//   '.json': 'json',
//   '.toml': 'toml',
//   '.text': 'text',
//   '.wasm': 'wasm',
//   '.napi': 'napi',
//   '.node': 'napi'
// };

// export function guessIdLoader(id: string): string {
//   return ExtToLoader[path.extname(id).toLowerCase()] || 'js';
// }

// export function transformQuery(context: any) {
//   const queryParamsObject: Record<string, string | boolean> = {};
//   context.query.forEach(([param, value]: string[]) => {
//     queryParamsObject[param] = value;
//   });
//   const transformQuery = querystring.stringify(queryParamsObject);
//   context.resolvedPath = `${context.resolvedPath}?${transformQuery}`;
// }

export function convertEnforceToPriority(value: 'pre' | 'post' | undefined) {
  const defaultPriority = 100;
  const enforceToPriority = {
    pre: 101,
    post: 98
  };

  return enforceToPriority[value!] !== undefined
    ? enforceToPriority[value!]
    : defaultPriority;
}

export function convertWatchEventChange(
  value: WatchChangeEvents
): WatchChangeEvents {
  const watchEventChange = {
    Added: 'create',
    Updated: 'update',
    Removed: 'delete'
  } as unknown as { [key in WatchChangeEvents]: WatchChangeEvents };

  return watchEventChange[value];
}

export function getContentValue(content: any): string {
  return encodeStr(typeof content === 'string' ? content : content!.code);
}

export function isString(variable: unknown): variable is string {
  return typeof variable === 'string';
}

export function isObject(variable: unknown): variable is object {
  return typeof variable === 'object' && variable !== null;
}

export function customParseQueryString(url: string | null) {
  if (!url) {
    return [];
  }

  const queryString = url.split('?')[1];

  const parsedParams = querystring.parse(queryString);
  const paramsArray = [];

  for (const key in parsedParams) {
    paramsArray.push([key, parsedParams[key] as string]);
  }

  return paramsArray as [string, string][];
}

export const VITE_PLUGIN_DEFAULT_MODULE_TYPE =
  'VITE_PLUGIN_DEFAULT_MODULE_TYPE';

export const CSS_LANGS_RES: [RegExp, string][] = [
  [/\.(css)(?:$|\?)/, 'css'],
  [/\.(less)(?:$|\?)/, 'less'],
  [/\.(scss|sass)(?:$|\?)/, 'sass'],
  [/\.(styl|stylus)(?:$|\?)/, 'stylus']
];

export const JS_LANGS_RES: [RegExp, string][] = [
  [/\.(js|mjs|cjs|)(?:$|\?)/, 'js'],
  // jsx
  [/\.(jsx)(?:$|\?)/, 'jsx'],
  // ts
  [/\.(ts|cts|mts)(?:$|\?)/, 'ts'],
  // tsx
  [/\.(tsx)(?:$|\?)/, 'tsx']
];

export const stringifyQuery = (query: [string, string][]) => {
  if (!query.length) {
    return '';
  }

  let queryStr = '';

  for (const [key, value] of query) {
    queryStr += `${key}${value ? `=${value}` : ''}&`;
  }

  return `${queryStr.slice(0, -1)}`;
};

export function formatId(id: string, query: [string, string][]): string {
  if (!query.length) {
    return id;
  }

  return `${id}?${stringifyQuery(query)}`;
}

export function getCssModuleType(id: string): string | null {
  for (const [reg, lang] of CSS_LANGS_RES) {
    if (reg.test(id)) {
      return lang;
    }
  }

  return null;
}

export function getJsModuleType(id: string): string | null {
  for (const [reg, lang] of JS_LANGS_RES) {
    if (reg.test(id)) {
      return lang;
    }
  }

  return null;
}

export function formatLoadModuleType(
  id: string,
  defaultModuleType = 'js'
): string {
  const cssModuleType = getCssModuleType(id);

  if (cssModuleType) {
    return cssModuleType;
  }

  const jsModuleType = getJsModuleType(id);

  if (jsModuleType) {
    return jsModuleType;
  }

  return defaultModuleType;
}

export function formatTransformModuleType(id: string): string {
  const cssModuleType = getCssModuleType(id);

  if (cssModuleType) {
    return cssModuleType;
  }

  return 'js';
}

// normalize invalid characters in id, for example: \0
// because characters like \0 have issues when passing to Farm's rust compiler
export function encodeStr(str: string): string {
  const result = addSlashes(str);

  // revert \\n to \n and \\t to \t and \" to ".
  return result
    .replace(/\\([ntrbf"])/g, (_, char) => {
      switch (char) {
        case 'n':
          return '\n';
        case 't':
          return '\t';
        case 'r':
          return '\r';
        case 'b':
          return '\b';
        case 'f':
          return '\f';
        default:
          return char;
      }
    })
    .replace(/\\(")/g, '$1');
}

export function decodeStr(str: string): string {
  return removeSlashes(str);
}
