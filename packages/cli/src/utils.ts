import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import Module from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import walkdir from 'walkdir';
import type { start, build } from '@farmfe/core';
import spawn from 'cross-spawn';

interface installProps {
  cwd: string; // 项目路径
  package: string; // 包管理器 yarn 或者 npm
}

export const TEMPLATES_DIR = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
  'templates'
);

export function copyFiles(
  source: string,
  dest: string,
  callback?: (content: string) => string
): void {
  walkdir(source, { sync: true }, (p, stat) => {
    if (stat.isFile()) {
      const content = readFileSync(p).toString();
      const newContent = callback?.(content) ?? content;

      const relativePath = path.relative(source, p);
      const destPath = path.join(dest, relativePath);

      if (!existsSync(path.dirname(destPath))) {
        mkdirSync(path.dirname(destPath), { recursive: true });
      }

      writeFileSync(destPath, newContent);
    }
  });

  if (!existsSync(path.join(dest, '.gitignore'))) {
    writeFileSync(
      path.join(dest, '.gitignore'),
      `
node_modules
*.farm`
    );
  }
}

export function resolveCore(cwd: string): Promise<{
  start: typeof start;
  build: typeof build;
}> {
  const require = Module.createRequire(path.join(cwd, 'package.json'));
  const farmCorePath = require.resolve('@farmfe/core');

  if (process.platform === 'win32') {
    return import(pathToFileURL(farmCorePath).toString());
  }

  return import(farmCorePath);
}
export async function install(options: installProps): Promise<void> {
  const cwd = options.cwd;
  return new Promise((resolve, reject) => {
    const command = options.package;
    const args = ['install'];

    const child = spawn(command, args, {
      cwd,
      stdio: 'inherit',
    });

    child.once('close', (code: number) => {
      if (code !== 0) {
        reject({
          command: `${command} ${args.join(' ')}`,
        });
        return;
      }
      resolve();
    });
    child.once('error', reject);
  });
}
/**
 * 用于规范化目标路径
 * @param {string |undefined} targetDir
 * @returns
 */
export function formatTargetDir(targetDir: string | undefined) {
  return targetDir?.trim().replace(/\/+$/g, '');
}
