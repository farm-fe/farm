import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import readline from 'node:readline';
import { fileURLToPath } from 'node:url';
import type {
  build,
  clean,
  preview,
  start,
  startRefactorCli,
  watch
} from '@farmfe/core';
import { Logger } from '@farmfe/core';
import spawn from 'cross-spawn';
import walkdir from 'walkdir';

import type { CleanOptions, GlobalCliOptions } from './types.js';

const logger = new Logger();
interface installProps {
  cwd: string;
  package: string;
}

export const TEMPLATES_DIR = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
  'templates'
);

export async function resolveCore(): Promise<{
  start: typeof start;
  build: typeof build;
  watch: typeof watch;
  preview: typeof preview;
  clean: typeof clean;
  startRefactorCli: typeof startRefactorCli;
  buildRefactorCli: typeof build;
}> {
  try {
    return import('@farmfe/core');
  } catch (err) {
    logger.error(
      `Cannot find @farmfe/core module, Did you successfully install: \n${err.stack},`
    );
    process.exit(1);
  }
}

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
  return targetDir?.trim()?.replace(/\/+$/g, '');
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

export function cleanOptions(options: GlobalCliOptions) {
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
  options: GlobalCliOptions
): GlobalCliOptions {
  const resolveOptions = { ...options };
  filterDuplicateOptions(resolveOptions);
  return cleanOptions(resolveOptions);
}

export function getConfigPath(root: string, configPath: string) {
  return path.resolve(root, configPath ?? '');
}

export async function handleAsyncOperationErrors<T>(
  asyncOperation: Promise<T>,
  errorMessage: string
) {
  try {
    await asyncOperation;
  } catch (error) {
    logger.error(`${errorMessage}:\n${error.stack}`);
    process.exit(1);
  }
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

export function resolveRootPath(rootPath = '') {
  return rootPath && path.isAbsolute(rootPath)
    ? rootPath
    : path.resolve(process.cwd(), rootPath);
}

export function resolveCliConfig(
  root: string,
  options: GlobalCliOptions & CleanOptions
) {
  root = resolveRootPath(root);
  const configPath = getConfigPath(root, options.config);
  return {
    root,
    configPath
  };
}
