import fs from 'node:fs';
import os from 'node:os';
import path, { dirname } from 'node:path';
import readline from 'node:readline';
import { fileURLToPath } from 'node:url';
import { Config, OutputConfig } from '../types/binding.js';
import { cleanUrl } from './url.js';
// @ts-ignore import packageJson from '../../package.json';

const __dirname = dirname(fileURLToPath(import.meta.url));

const splitRE = /\r?\n/;

export const FARM_TARGET_NODE_ENVS = [
  'node',
  'node16',
  'node-legacy',
  'node-next'
];
export const FARM_TARGET_BROWSER_ENVS = [
  'browser',
  'browser-legacy',
  'browser-es2015',
  'browser-es2017',
  'browser-esnext'
];

export const FARM_TARGET_LIBRARY_ENVS = [
  'library'
  // 'library-node',
  // 'library-browser'
];

export function isObject(value: unknown): value is Record<string, unknown> {
  return Object.prototype.toString.call(value) === '[object Object]';
}

export function isArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

export function isEmptyObject<T extends object>(obj: T): boolean {
  if (!obj) return true;
  return Reflect.ownKeys(obj).length === 0;
}

export const isUndefined = (obj: any): obj is undefined =>
  typeof obj === 'undefined';
export const isString = (val: any): val is string => typeof val === 'string';
export const isNumber = (val: any): val is number => typeof val === 'number';
export const isEmpty = (array: any): boolean => !(array && array.length > 0);
export const isSymbol = (val: any): val is symbol => typeof val === 'symbol';

export const isWindows = os.platform() === 'win32';

export function pad(source: string, n = 2): string {
  const lines = source.split(splitRE);
  return lines.map((l) => ` `.repeat(n) + l).join(`\n`);
}

export function clearScreen() {
  try {
    const repeatCount = process.stdout.rows - 2;
    const blank = repeatCount > 0 ? '\n'.repeat(repeatCount) : '';
    console.log(blank);
    readline.cursorTo(process.stdout, 0, 0);
    readline.clearScreenDown(process.stdout);
  } catch (error) {
    console.error('Failed to clear screen:', error);
  }
}

export const version = JSON.parse(
  fs.readFileSync(path.resolve(__dirname, '../../package.json')).toString()
).version;

export const VOLUME_RE = /^[A-Z]:/i;

export const FS_PREFIX = `/@fs/`;

export function normalizePath(id: string): string {
  return path.posix.normalize(isWindows ? id.replace(/\\/g, '/') : id);
}

export function fsPathFromId(id: string): string {
  const fsPath = normalizePath(
    id.startsWith(FS_PREFIX) ? id.slice(FS_PREFIX.length) : id
  );
  return fsPath[0] === '/' || VOLUME_RE.test(fsPath) ? fsPath : `/${fsPath}`;
}

export function fsPathFromUrl(url: string): string {
  return fsPathFromId(cleanUrl(url));
}

export function arraify<T>(target: T | T[]): T[] {
  return Array.isArray(target) ? target : [target];
}

export function getFileSystemStats(file: string): fs.Stats | undefined {
  try {
    return fs.statSync(file, { throwIfNoEntry: false });
  } catch (error) {
    console.error(`Error accessing file ${file}:`, error);
    return undefined;
  }
}

/**
 * Null or whatever
 */
export type Nullable<T> = T | null | undefined;

/**
 * Array, or not yet
 */
export type ArrayAble<T> = T | Array<T>;

export function toArray<T>(array?: Nullable<ArrayAble<T>>): Array<T> {
  return array ? (Array.isArray(array) ? array : [array]) : [];
}

export function mergeObjects<
  T extends Record<string, any>,
  U extends Record<string, any>
>(obj1: T, obj2: U): T & U {
  const merged: Record<string, any> = { ...obj1 };

  Object.keys(obj2).forEach((key) => {
    if (Object.prototype.hasOwnProperty.call(obj2, key)) {
      if (
        merged.hasOwnProperty(key) &&
        typeof obj2[key] === 'object' &&
        !Array.isArray(obj2[key])
      ) {
        merged[key] = mergeObjects(merged[key], obj2[key]);
      } else {
        merged[key] = obj2[key];
      }
    }
  });

  return merged as T & U;
}

export async function asyncFlatten<T>(arr: T[]): Promise<T[]> {
  do {
    arr = (await Promise.all(arr)).flat(Infinity) as any;
  } while (arr.some((v: any) => v?.then));
  return arr;
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// prevent node experimental warning
export function preventExperimentalWarning() {
  const defaultEmit = process.emit;
  process.emit = function (...args: any[]) {
    if (args[1].name === 'ExperimentalWarning') {
      return undefined;
    }
    return defaultEmit.call(this, ...args);
  };
}

export function mapTargetEnvValue(config: Config['config']) {
  if (FARM_TARGET_NODE_ENVS.includes(config.output.targetEnv)) {
    config.output.targetEnv = 'node';
  } else if (FARM_TARGET_BROWSER_ENVS.includes(config.output.targetEnv)) {
    config.output.targetEnv = 'browser';
  } else {
    if (FARM_TARGET_LIBRARY_ENVS.includes(config.output.targetEnv)) {
      return;
    }

    config.output.targetEnv = 'library';
  }
}

export function tryStatSync(file: string): fs.Stats | undefined {
  try {
    return fs.statSync(file, { throwIfNoEntry: false });
  } catch {}
}

export function formatTime(time: number, format: 'ms' | 's' = 'ms'): string {
  switch (format) {
    case 's':
      return `${Math.floor(time) / 1000}s`;
    case 'ms':
    default:
      return `${Math.floor(time)}ms`;
  }
}

export function arrayEqual(a: any[], b: any[]): boolean {
  if (a === b) return true;
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

export function isNodeEnv(env: OutputConfig['targetEnv']): boolean {
  return /^(node|library)(?!-browser)/.test(env);
}

export function getValidPublicPath(publicPath = '/'): string {
  let validPublicPath = '';

  if (publicPath.startsWith('/')) {
    validPublicPath = publicPath;
  } else if (publicPath.startsWith('.')) {
    validPublicPath = normalizePath(path.join('/', publicPath));
  }

  return validPublicPath;
}
