import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import readline from 'node:readline';
import { fileURLToPath } from 'node:url';
import walkdir from 'walkdir';
import spawn from 'cross-spawn';

import type { GlobalFarmCLIOptions } from './types.js';

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

export async function install(options: installProps): Promise<void> {
  const cwd = options.cwd;
  return new Promise((resolve, reject) => {
    const command = options.package;
    const args = ['install'];

    const child = spawn(command, args, {
      cwd,
      stdio: 'inherit'
    });

    child.once('close', (code: number) => {
      if (code !== 0) {
        reject({
          command: `${command} ${args.join(' ')}`
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

/**
 * filter duplicate item in options
 */
export function filterDuplicateOptions<T>(options: T) {
  for (const [key, value] of Object.entries(options)) {
    if (Array.isArray(value)) {
      options[key as keyof T] = value[value.length - 1];
    }
  }
}

/**
 * clear command screen
 */
export function clearScreen() {
  const repeatCount = process.stdout.rows - 2;
  const blank = repeatCount > 0 ? '\n'.repeat(repeatCount) : '';
  console.log(blank);
  readline.cursorTo(process.stdout, 0, 0);
  readline.clearScreenDown(process.stdout);
}

export function cleanOptions(options: GlobalFarmCLIOptions) {
  const resolveOptions = { ...options };

  delete resolveOptions['--'];
  delete resolveOptions.m;
  delete resolveOptions.c;
  delete resolveOptions.w;
  delete resolveOptions.l;
  delete resolveOptions.lazy;
  delete resolveOptions.mode;
  delete resolveOptions.base;
  delete resolveOptions.config;
  delete resolveOptions.clearScreen;

  return resolveOptions;
}

export function resolveCommandOptions(
  options: GlobalFarmCLIOptions
): GlobalFarmCLIOptions {
  const resolveOptions = { ...options };
  filterDuplicateOptions(resolveOptions);
  return cleanOptions(resolveOptions);
}

export function getConfigPath(configPath: string) {
  return path.join(process.cwd(), configPath ?? '');
}
