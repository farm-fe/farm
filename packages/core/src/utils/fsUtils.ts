import { exec } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import fse from 'fs-extra';
import { ResolvedUserConfig, normalizePublicDir } from '../config/index.js';
import { colors } from './color.js';
import { isWindows, normalizePath, tryStatSync } from './share.js';

function isDirectory(path: string): boolean {
  const stat = tryStatSync(path);
  return stat?.isDirectory() ?? false;
}

function tryResolveRealFile(
  file: string,
  preserveSymlinks?: boolean
): string | undefined {
  const stat = tryStatSync(file);
  if (stat?.isFile()) return getRealPath(file, preserveSymlinks);
}

function tryResolveRealFileWithExtensions(
  filePath: string,
  extensions: string[],
  preserveSymlinks?: boolean
): string | undefined {
  for (const ext of extensions) {
    const res = tryResolveRealFile(filePath + ext, preserveSymlinks);
    if (res) return res;
  }
}

function tryResolveRealFileOrType(
  file: string,
  preserveSymlinks?: boolean
): { path?: string; type: 'directory' | 'file' } | undefined {
  const fileStat = tryStatSync(file);
  if (fileStat?.isFile()) {
    return { path: getRealPath(file, preserveSymlinks), type: 'file' };
  }
  if (fileStat?.isDirectory()) {
    return { type: 'directory' };
  }
  return;
}
const windowsNetworkMap = new Map();

function windowsMappedRealpathSync(path: string) {
  const realPath = fs.realpathSync.native(path);
  if (realPath.startsWith('\\\\')) {
    for (const [network, volume] of windowsNetworkMap) {
      if (realPath.startsWith(network))
        return realPath.replace(network, volume);
    }
  }
  return realPath;
}
function optimizeSafeRealPathSync() {
  // Skip if using Node <18.10 due to MAX_PATH issue: https://github.com/vitejs/vite/issues/12931
  const nodeVersion = process.versions.node.split('.').map(Number);
  if (nodeVersion[0] < 18 || (nodeVersion[0] === 18 && nodeVersion[1] < 10)) {
    safeRealpathSync = fs.realpathSync;
    return;
  }
  // Check the availability `fs.realpathSync.native`
  // in Windows virtual and RAM disks that bypass the Volume Mount Manager, in programs such as imDisk
  // get the error EISDIR: illegal operation on a directory
  try {
    fs.realpathSync.native(path.resolve('./'));
  } catch (error) {
    if (error.message.includes('EISDIR: illegal operation on a directory')) {
      safeRealpathSync = fs.realpathSync;
      return;
    }
  }
  exec('net use', (error, stdout) => {
    if (error) return;
    const lines = stdout.split('\n');
    // OK           Y:        \\NETWORK\Foo         Microsoft Windows Network
    // OK           Z:        \\NETWORK\Bar         Microsoft Windows Network
    for (const line of lines) {
      const m = line.match(parseNetUseRE);
      if (m) windowsNetworkMap.set(m[3], m[2]);
    }
    if (windowsNetworkMap.size === 0) {
      safeRealpathSync = fs.realpathSync.native;
    } else {
      safeRealpathSync = windowsMappedRealpathSync;
    }
  });
}

const parseNetUseRE = /^(\w+)? +(\w:) +([^ ]+)\s/;
let firstSafeRealPathSyncRun = false;

function windowsSafeRealPathSync(path: string): string {
  if (!firstSafeRealPathSyncRun) {
    optimizeSafeRealPathSync();
    firstSafeRealPathSyncRun = true;
  }
  return fs.realpathSync(path);
}

// `fs.realpathSync.native` resolves differently in Windows network drive,
// causing file read errors. skip for now.
// https://github.com/nodejs/node/issues/37737
export let safeRealpathSync = isWindows
  ? windowsSafeRealPathSync
  : fs.realpathSync.native;

function getRealPath(resolved: string, preserveSymlinks?: boolean): string {
  if (!preserveSymlinks) {
    resolved = safeRealpathSync(resolved);
  }
  return normalizePath(resolved);
}

export const commonFsUtils = {
  existsSync: fs.existsSync,
  isDirectory,

  tryResolveRealFile,
  tryResolveRealFileWithExtensions,
  tryResolveRealFileOrType
};

export async function readFileIfExists(value?: string | Buffer | any[]) {
  if (typeof value === 'string') {
    return fse.readFile(path.resolve(value)).catch(() => value);
  }
  return value;
}

export async function findNodeModulesRecursively(
  rootPath: string
): Promise<string[]> {
  const result: string[] = [];

  async function traverse(currentPath: string) {
    const items = await fse.readdir(currentPath);
    for (const item of items) {
      const fullPath = path.join(currentPath, item);
      const stats = await fse.stat(fullPath);

      if (stats.isDirectory()) {
        if (item === 'node_modules') {
          result.push(fullPath);
        } else {
          await traverse(fullPath);
        }
      }
    }
  }

  await traverse(rootPath);
  return result;
}

export async function copyPublicDirectory(
  resolvedUserConfig: ResolvedUserConfig
): Promise<void> {
  const absPublicDirPath = normalizePublicDir(
    resolvedUserConfig.root,
    resolvedUserConfig.publicDir
  );

  try {
    if (await fse.pathExists(absPublicDirPath)) {
      const files = await fse.readdir(absPublicDirPath);
      const outputPath = resolvedUserConfig.compilation.output.path;
      for (const file of files) {
        const publicFile = path.join(absPublicDirPath, file);
        const destFile = path.join(outputPath, file);

        if (await fse.pathExists(destFile)) {
          continue;
        }
        await fse.copy(publicFile, destFile);
      }

      resolvedUserConfig.logger.info(
        `Public directory resources copied ${colors.bold(
          colors.green('successfully')
        )}.`
      );
    }
  } catch (error) {
    resolvedUserConfig.logger.error(
      `Error copying public directory: ${error.message}`
    );
  }
}
