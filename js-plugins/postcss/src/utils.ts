import { UserConfig } from '@farmfe/core';
import fs from 'fs';
import { createRequire } from 'module';
import path from 'path';
import fsp from 'node:fs/promises';

const __require = createRequire(import.meta.url);
const publicFilesMap = new WeakMap<UserConfig, Set<string>>();

export const { name: pluginName } = __require('../../package.json');

export function getPostcssImplementation(implementation?: string) {
  let resolvedImplementation;
  if (!implementation || typeof implementation === 'string') {
    const lessImplPkg = implementation || 'postcss';
    try {
      resolvedImplementation = __require(lessImplPkg);
    } catch (e) {
      throwError('Implementation', e);
    }
  }
  return resolvedImplementation;
}

export function throwError(type: string, error: Error) {
  console.error(`[${pluginName} ${type} Error] ${error}`);
}

export async function tryRead(filename: string) {
  try {
    return await fs.promises.readFile(filename, 'utf-8');
  } catch (e) {
    throwError('readFile', e);
  }
}

export async function checkPublicFile(url: string, config: UserConfig) {
  // note if the file is in /public, the resolver would have returned it
  // as-is so it's not going to be a fully resolved path.
  const { publicDir } = config;
  if (!publicDir || url[0] !== '/') {
    return;
  }
  await initPublicFiles(config);
  const fileName = cleanUrl(url);
  const publicFiles = getPublicFiles(config);
  if (publicFiles) {
    return publicFiles.has(fileName)
      ? normalizePath(path.join(publicDir, fileName))
      : undefined;
  }

  const publicFile = normalizePath(path.join(publicDir, fileName));
  if (!publicFile.startsWith(withTrailingSlash(publicDir))) {
    // can happen if URL starts with '../'
    return;
  }
  return fs.existsSync(publicFile) ? publicFile : undefined;
}

const postfixRE = /[?#].*$/;
export function cleanUrl(url: string): string {
  return url.replace(postfixRE, '');
}

export const ERR_SYMLINK_IN_RECURSIVE_READDIR =
  'ERR_SYMLINK_IN_RECURSIVE_READDIR';
export async function recursiveReaddir(dir: string): Promise<string[]> {
  if (!fs.existsSync(dir)) {
    return [];
  }
  let dirents: fs.Dirent[];
  try {
    dirents = await fsp.readdir(dir, { withFileTypes: true });
  } catch (e) {
    if (e.code === 'EACCES') {
      // Ignore permission errors
      return [];
    }
    throw e;
  }
  if (dirents.some((dirent) => dirent.isSymbolicLink())) {
    const err: any = new Error(
      'Symbolic links are not supported in recursiveReaddir'
    );
    err.code = ERR_SYMLINK_IN_RECURSIVE_READDIR;
    throw err;
  }
  const files = await Promise.all(
    dirents.map((dirent) => {
      const res = path.resolve(dir, dirent.name);
      return dirent.isDirectory() ? recursiveReaddir(res) : normalizePath(res);
    })
  );
  return files.flat(1);
}

const windowsSlashRE = /\\/g;
export function slash(p: string): string {
  return p.replace(windowsSlashRE, '/');
}

export function normalizePath(id: string): string {
  return path.posix.normalize(isWindows ? slash(id) : id);
}

export const isWindows =
  typeof process !== 'undefined' && process.platform === 'win32';

export async function initPublicFiles(
  config: UserConfig
): Promise<Set<string> | undefined> {
  let fileNames: string[];
  try {
    fileNames = await recursiveReaddir(config.publicDir);
  } catch (e) {
    if (e.code === ERR_SYMLINK_IN_RECURSIVE_READDIR) {
      return;
    }
    throw e;
  }
  const publicFiles = new Set(
    fileNames.map((fileName) => fileName.slice(config.publicDir.length))
  );
  publicFilesMap.set(config, publicFiles);
  return publicFiles;
}

function getPublicFiles(config: UserConfig): Set<string> | undefined {
  return publicFilesMap.get(config);
}

export function withTrailingSlash(path: string): string {
  if (path[path.length - 1] !== '/') {
    return `${path}/`;
  }
  return path;
}
