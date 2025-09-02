import { readFileSync } from 'node:fs';
import { isAbsolute, resolve } from 'node:path';

import { Logger } from '../utils/logger.js';

import type {
  CleanOptions,
  CliBuildOptions,
  CliServerOptions,
  GlobalCliOptions
} from '../config/types.js';
import type { build, clean, preview, start, watch } from '../index.js';

const logger = new Logger();

/**
 *
 * @returns  {Promise<{ start: typeof start, build: typeof build, watch: typeof watch, preview: typeof preview, clean: typeof clean }>}
 * A promise that resolves to an object containing the core functionalities:
 *   - `start`: Compile the project in dev mode and serve it with farm dev server'.
 *   - `build`: compile the project in production mode'.
 *   - `watch`: watch file change'.
 *   - `preview`: compile the project in watch mode'.
 *   - `clean`: Clean up the cache built incrementally'.
 */
export async function resolveCore(): Promise<{
  start: typeof start;
  build: typeof build;
  watch: typeof watch;
  preview: typeof preview;
  clean: typeof clean;
}> {
  try {
    return import('../index.js');
  } catch (err) {
    logger.error(
      `Cannot find @farmfe/core module, Did you successfully install: \n${err.stack},`,
      { exit: true }
    );
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
/**
 * @param options The cli passes parameters
 * @returns Remove parameters that are not required
 */
export function cleanOptions(
  options: GlobalCliOptions & CliServerOptions & CliBuildOptions
) {
  const resolveOptions = { ...options };

  delete resolveOptions['--'];
  delete resolveOptions.m;
  delete resolveOptions.c;
  delete resolveOptions.l;
  delete resolveOptions.lazy;
  delete resolveOptions.mode;
  delete resolveOptions.base;
  delete resolveOptions.config;
  delete resolveOptions.clearScreen;

  return resolveOptions;
}

/**
 *
 * @param options cli parameters
 * @returns resolve command options
 */
export function resolveCommandOptions(
  options: GlobalCliOptions & CliServerOptions
): GlobalCliOptions & CliServerOptions {
  const resolveOptions = { ...options };
  filterDuplicateOptions(resolveOptions);
  return cleanOptions(resolveOptions);
}

/**
 *
 * @param root root path
 * @param configPath  config path
 * @returns config path absolute path
 */
export function getConfigPath(root: string, configPath: string) {
  return resolve(root, configPath ?? '');
}

/**
 *
 * @param asyncOperation The asynchronous operation to be executed.
 * @param errorMessage The error message to log if the operation fails.
 */
export async function handleAsyncOperationErrors<T>(
  asyncOperation: Promise<T>,
  errorMessage: string
) {
  try {
    await asyncOperation;
  } catch (error) {
    logger.error(`${errorMessage}:\n${error.stack}`, { exit: true });
  }
}

/**
 *
 * @param rootPath root path
 * @returns absolute path
 */
export function resolveRootPath(rootPath = '') {
  return rootPath && isAbsolute(rootPath)
    ? rootPath
    : resolve(process.cwd(), rootPath);
}

/**
 *
 * @param root root path
 * @param options cli parameters
 * @returns
 *  - root root path
 *  - configPath
 */
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

const { version } = JSON.parse(
  readFileSync(new URL('../../package.json', import.meta.url)).toString()
);

export const VERSION = version;
