import { readFileSync } from 'node:fs';
import path from 'node:path';
import type { build, clean, preview, start, watch } from '@farmfe/core';
import { Logger } from '@farmfe/core';

import type {
  CleanOptions,
  CliBuildOptions,
  CliServerOptions,
  GlobalCliOptions
} from './types.js';

const logger = new Logger();

export async function resolveCore(): Promise<{
  start: typeof start;
  build: typeof build;
  watch: typeof watch;
  preview: typeof preview;
  clean: typeof clean;
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

export function cleanOptions(
  options: GlobalCliOptions & CliServerOptions & CliBuildOptions
) {
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

export const { version } = JSON.parse(
  readFileSync(new URL('../package.json', import.meta.url)).toString()
);
