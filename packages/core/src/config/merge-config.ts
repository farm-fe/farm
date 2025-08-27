import path, { isAbsolute } from 'node:path';

import { isArray, isObject, isString } from '../utils/share.js';
import { CompilationMode } from './env.js';
import { FarmCliOptions, UserConfig } from './types.js';

export function mergeConfig<T extends Record<string, any>>(
  userConfig: T,
  target?: T
): T {
  const result: Record<string, any> = { ...userConfig };
  for (const key of Object.keys(target || {})) {
    const left = result[key];
    const right = target[key];

    if (right === null || right === undefined) {
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

    if (isObject(right)) {
      result[key] = mergeConfig({}, right);
      continue;
    }

    result[key] = right;
  }

  return result as T;
}

export function mergeFarmCliConfig(
  cliOption: FarmCliOptions & UserConfig,
  target: UserConfig,
  mode: CompilationMode
): UserConfig {
  let left: UserConfig = {};

  (
    [
      'clearScreen',
      'mode',
      'compilation',
      'envDir',
      'envPrefix',
      'watch',
      'plugins',
      'publicDir',
      'server',
      'vitePlugins'
    ] satisfies (keyof UserConfig)[]
  ).forEach((key: keyof (FarmCliOptions & UserConfig)) => {
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
        target.root = path.resolve(cliRoot);
      } else {
        target.root = cliRoot;
      }
    } else {
      target.root = path.resolve('.');
    }

    if (configRootPath) {
      target.root = configRootPath;
    }

    if (
      target.root &&
      !isAbsolute(target.root) &&
      cliOption.configFile !== false
    ) {
      const resolvedRoot = path.resolve(cliOption.configFile, target.root);
      target.root = resolvedRoot;
    }
  }

  if (
    isString(cliOption.server?.host) ||
    typeof cliOption.server?.host === 'boolean'
  ) {
    left = mergeConfig(left, { server: { host: cliOption.host } });
  }

  if (typeof cliOption.compilation?.minify === 'boolean') {
    left = mergeConfig(left, { compilation: { minify: cliOption.minify } });
  }

  if (cliOption.compilation?.output?.path) {
    left = mergeConfig(left, {
      compilation: { output: { path: cliOption.outDir } }
    });
  }

  if (cliOption.mode) {
    left = mergeConfig(left, {
      compilation: {
        mode: mode ?? (cliOption.mode as UserConfig['compilation']['mode'])
      }
    });
  }

  if (cliOption.server?.port) {
    left = mergeConfig(left, {
      server: {
        port: cliOption.port
      }
    });
  }

  if (cliOption.server?.https) {
    left = mergeConfig(left, {
      server: {
        https: cliOption.https
      }
    });
  }

  if (cliOption.compilation?.sourcemap) {
    left = mergeConfig(left, {
      compilation: { sourcemap: cliOption.sourcemap }
    });
  }

  return mergeConfig(left, target);
}
