// import path from 'node:path';
import * as querystring from 'node:querystring';

export type WatchChangeEvents = 'create' | 'update' | 'delete';

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

export const DEFAULT_FILTERS = ['!node_modules'];

export const FARM_CSS_MODULE_SUFFIX = /\.FARM_CSS_MODULES(?:$|\?)/;

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

export function formatLoadModuleType(id: string): string {
  const cssModuleType = getCssModuleType(id);

  if (cssModuleType) {
    return cssModuleType;
  }

  const jsModuleType = getJsModuleType(id);

  if (jsModuleType) {
    return jsModuleType;
  }

  return 'js';
}

export function formatTransformModuleType(id: string): string {
  return formatLoadModuleType(id);
}

// normalize invalid characters in id, for example: \0
// because characters like \0 have issues when passing to Farm's rust compiler
export function encodeStr(str: string): string {
  return str.replace(/\0/g, '\\0');
}

export function decodeStr(str: string): string {
  return str.replace(/\\0/g, '\0');
}

export function deleteUndefinedPropertyDeeply(obj: any) {
  if (typeof obj !== 'object') {
    return;
  }

  for (const key in obj) {
    if (!Object.prototype.hasOwnProperty.call(obj, key)) {
      continue;
    }

    if (Array.isArray(obj[key])) {
      obj[key] = obj[key].filter((item: any) => item !== undefined);
    } else if (obj[key] === undefined) {
      delete obj[key];
    } else if (typeof obj[key] === 'object') {
      deleteUndefinedPropertyDeeply(obj[key]);
    }
  }
}

export function throwIncompatibleError(
  pluginName: string,
  readingObject: string,
  allowedKeys: string[],
  key: string | number | symbol
): never {
  throw new Error(
    `Vite plugin '${pluginName}' is not compatible with Farm for now. Because it uses ${readingObject}['${String(
      key
    )}'] which is not supported by Farm. Current supported keys are: ${allowedKeys.join(
      ','
    )}`
  );
}
