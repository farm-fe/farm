import { throwError as t } from '@farmfe/core';
import fs from 'fs';
import { createRequire } from 'module';

const __require = createRequire(import.meta.url);

export const { name: pluginName } = __require('../../package.json');

export function throwError(type: string, error: Error) {
  t(pluginName, type, error);
}
export async function tryRead(filename: string) {
  try {
    return await fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('read', e);
  }
}

// The same as Vite
// in unix, scss might append `location.href` in environments that shim `location`
// see https://github.com/sass/dart-sass/issues/710
export function cleanScssBugUrl(url: string) {
  if (
    // check bug via `window` and `location` global
    typeof window !== 'undefined' &&
    typeof location !== 'undefined' &&
    typeof location?.href === 'string'
  ) {
    const prefix = location.href.replace(/\/$/, '');
    return url.replace(prefix, '');
  } else {
    return url;
  }
}
