import os from 'node:os';
import readline from 'node:readline';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

/* eslint-disable @typescript-eslint/no-use-before-define */
export function isObject(value: unknown): value is Record<string, unknown> {
  return Object.prototype.toString.call(value) === '[object Object]';
}

export function isArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

export function isEmptyObject<T extends object>(obj: T): boolean {
  return Reflect.ownKeys(obj).length === 0;
}
export const isUndefined = (obj: any): obj is undefined =>
  typeof obj === 'undefined';
export const isString = (val: any): val is string => typeof val === 'string';
export const isNumber = (val: any): val is number => typeof val === 'number';
export const isEmpty = (array: any): boolean => !(array && array.length > 0);
export const isSymbol = (val: any): val is symbol => typeof val === 'symbol';

export const isWindows = os.platform() === 'win32';

export function clearScreen() {
  const repeatCount = process.stdout.rows - 2;
  const blank = repeatCount > 0 ? '\n'.repeat(repeatCount) : '';
  console.log(blank);
  readline.cursorTo(process.stdout, 0, 0);
  readline.clearScreenDown(process.stdout);
}

export function normalizePath(id: string): string {
  return path.posix.normalize(id);
}

export function arraify<T>(target: T | T[]): T[] {
  return Array.isArray(target) ? target : [target];
}

export function getFileSystemStats(file: string): fs.Stats | undefined {
  try {
    return fs.statSync(file, { throwIfNoEntry: false });
  } catch {
    // Ignore errors
  }
}

export function getDependenciesRecursive(config: any) {
  const content = fs.readFileSync(config.resolveConfigPath, 'utf-8');
  const dependencyRegex = /import\s.*?from\s['"](.+?)['"]/g;
  const requireRegex = /require\s*\(\s*['"](.+?)['"]\s*\)/g;
  const allDependencies = [];
  const dependencies = [];

  let match;
  while ((match = dependencyRegex.exec(content)) !== null) {
    dependencies.push(match[1]);
  }

  while ((match = requireRegex.exec(content)) !== null) {
    dependencies.push(match[1]);
  }

  for (const dependency of dependencies) {
    const dependencyPath = path.resolve(
      path.dirname(config.filePath),
      dependency
    );
    // 检查依赖项是否在项目内部，而不是在node_modules中
    if (!dependencyPath.includes('node_modules')) {
      allDependencies.push(dependencyPath);
      getDependenciesRecursive(dependencyPath);
    }
  }

  return allDependencies;
}

export function isInternalDependency(dependencyPath: string) {
  const projectRoot = path.resolve(__dirname);
  return dependencyPath.startsWith(projectRoot);
}

export async function importFresh(modulePath: string) {
  const cacheBustingModulePath = `${modulePath}?update=${Date.now()}`;
  if (process.platform === 'win32') {
    return (await import(pathToFileURL(cacheBustingModulePath).toString()))
      .default;
  } else {
    return (await import(cacheBustingModulePath)).default;
  }
}

export async function importFresh2(modulePath: string) {
  const filepath = path.resolve(modulePath);
  const fileContent = await fs.promises.readFile(filepath, 'utf8');
  const ext = path.extname(filepath);
  const extRegex = new RegExp(`\\${ext}$`);
  const newFilepath = `${filepath.replace(extRegex, '')}${Date.now()}${ext}`;

  await fs.promises.writeFile(newFilepath, fileContent);
  let module;
  if (process.platform === 'win32') {
    module = (await import(pathToFileURL(newFilepath).toString())).default;
  } else {
    module = (await import(newFilepath)).default;
  }
  fs.unlink(newFilepath, () => {});

  return module;
}
