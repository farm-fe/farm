import path, { isAbsolute } from 'node:path';
import { isString } from '../plugin/js/utils.js';
import { isArray, isObject } from '../utils/share.js';
import { FarmCliOptions, UserConfig } from './types.js';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function mergeConfig<T extends Record<string, any>>(
  userConfig: T,
  target: T
): T {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const result: Record<string, any> = { ...userConfig };
  for (const key of Object.keys(target)) {
    const left = result[key];
    const right = target[key];

    if (right === null || right === undefined) {
      continue;
    }

    if (left === null || left === undefined) {
      result[key] = right;
      continue;
    }

    if (isArray(left) || isArray(right)) {
      result[key] = [
        ...new Set([
          ...(isArray(left) ? left : []),
          ...(isArray(right) ? right : [])
        ])
      ];

      continue;
    }

    if (isObject(left) && isObject(right)) {
      result[key] = mergeConfig(left, right);
      continue;
    }

    result[key] = right;
  }

  return result as T;
}

export function mergeFarmCliConfig(
  cliOption: FarmCliOptions & UserConfig,
  target: UserConfig
): UserConfig {
  let left: UserConfig = {};

  (
    [
      'clearScreen',
      'compilation',
      'envDir',
      'envPrefix',
      'plugins',
      'publicDir',
      'server',
      'vitePlugins'
    ] satisfies (keyof UserConfig)[]
  ).forEach((key) => {
    const value = cliOption[key];
    if (value || typeof value === 'boolean') {
      left = mergeConfig(left, { [key]: cliOption[key] });
    }
  });

  {
    // root
    const configRootPath = target.root;

    if (cliOption.root) {
      const cliRoot = cliOption.root;

      if (!isAbsolute(cliRoot)) {
        target.root = path.resolve(process.cwd(), cliRoot);
      } else {
        target.root = cliRoot;
      }
    } else {
      target.root = process.cwd();
    }

    if (configRootPath) {
      target.root = configRootPath;
    }

    if (target.root && !isAbsolute(target.root)) {
      const resolvedRoot = path.resolve(cliOption.configPath, target.root);
      target.root = resolvedRoot;
    }
  }

  if (isString(cliOption.host) || typeof cliOption.host === 'boolean') {
    left = mergeConfig(left, { server: { host: cliOption.host } });
  }

  if (typeof cliOption.minify === 'boolean') {
    left = mergeConfig(left, { compilation: { minify: cliOption.minify } });
  }

  if (cliOption.outDir) {
    left = mergeConfig(left, {
      compilation: { output: { path: cliOption.outDir } }
    });
  }

  if (cliOption.port) {
    left = mergeConfig(left, {
      server: {
        port: cliOption.port
      }
    });
  }

  if (cliOption.mode) {
    left = mergeConfig(left, {
      compilation: {
        mode: cliOption.mode as UserConfig['compilation']['mode']
      }
    });
  }

  if (cliOption.https) {
    left = mergeConfig(left, {
      server: {
        https: cliOption.https
      }
    });
  }

  if (cliOption.sourcemap) {
    left = mergeConfig(left, {
      compilation: { sourcemap: cliOption.sourcemap }
    });
  }

  return mergeConfig(left, target);
}

export function initialCliOptions(options: FarmCliOptions): FarmCliOptions {
  return {
    ...options
  };
}
