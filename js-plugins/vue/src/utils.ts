import path from 'path';
import crypto from 'crypto';
import {
  PreProcessors,
  PreProcessorsType,
  outputData,
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
    query: queryObj,
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

export async function dynamicImportFromESM(moduleName: string) {
  try {
    const module = (await import(moduleName)) ?? {};
    return module.default ?? module;
  } catch (error) {
    throw error;
  }
}

export async function loadPreProcessor<T extends PreProcessorsType>(
  lang: T
): Promise<PreProcessors[T]> {
  try {
    const preProcessor = await dynamicImportFromESM(lang);
    return preProcessor;
  } catch (error) {
    if (error.code === 'ERR_MODULE_NOT_FOUND') {
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
    'modifyVars' in (preProcessor as PreProcessors[PreProcessorsType.less])
  );
}

export function isSass(
  preProcessor: unknown
): preProcessor is PreProcessors[PreProcessorsType.sass] {
  return (
    'renderSync' in (preProcessor as PreProcessors[PreProcessorsType.sass])
  );
}

export function isStyl(
  preProcessor: unknown
): preProcessor is PreProcessors[PreProcessorsType.stylus] {
  return (
    'convertCSS' in (preProcessor as PreProcessors[PreProcessorsType.stylus])
  );
}
