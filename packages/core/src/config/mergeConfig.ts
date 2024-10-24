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
  target: UserConfig,
  mode?: 'development' | 'production'
): UserConfig {
  let left: UserConfig = {};
  const options = initialCliOptions(cliOption);

  (
    [
      'clearScreen',
      'compilation',
      'envDir',
      'envPrefix',
      'timeUnit',
      'watch',
      'plugins',
      'publicDir',
      'server',
      'preview',
      'vitePlugins'
    ] satisfies (keyof UserConfig)[]
  ).forEach((key: keyof (FarmCliOptions & UserConfig)) => {
    const value = options[key];
    if (value || typeof value === 'boolean') {
      left = mergeConfig(left, { [key]: options[key] });
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

    if (target.root && !isAbsolute(target.root)) {
      const resolvedRoot = path.resolve(cliOption.configFile, target.root);
      target.root = resolvedRoot;
    }
  }

  if (
    isString(options.server?.host) ||
    typeof options.server?.host === 'boolean'
  ) {
    left = mergeConfig(left, { server: { host: options.host } });
  }

  if (typeof options.compilation.minify === 'boolean') {
    left = mergeConfig(left, { compilation: { minify: options.minify } });
  }

  if (options.compilation.output.outDir) {
    left = mergeConfig(left, {
      compilation: { output: { path: options.outDir } }
    });
  }

  if (options.server?.port) {
    left = mergeConfig(left, {
      server: {
        port: options.port
      }
    });
  }

  if (options.mode) {
    left = mergeConfig(left, {
      compilation: {
        mode: mode ?? (options.mode as UserConfig['compilation']['mode'])
      }
    });
  }

  if (options.server?.https) {
    left = mergeConfig(left, {
      server: {
        https: options.https
      }
    });
  }

  if (options.compilation?.sourcemap) {
    left = mergeConfig(left, {
      compilation: { sourcemap: options.sourcemap }
    });
  }
  if (options.timeUnit) {
    left = mergeConfig(left, {
      timeUnit: options.timeUnit
    });
  }

  return mergeConfig(left, target);
}

export function initialCliOptions(options: any): any {
  const { mode, watch } = options;

  const compilationOptions = options.compilation || {};
  const { minify, sourcemap, treeShaking } = compilationOptions;
  const { outDir, target, format } = compilationOptions.output || {};

  const input = compilationOptions.input
    ? Object.values(compilationOptions.input).filter(Boolean)
    : [];
  const hasInput = input.length > 0;

  const output: UserConfig['compilation']['output'] = {
    ...(outDir && { path: outDir }),
    ...(target && { targetEnv: target }),
    ...(format && { format })
  };

  const compilation: UserConfig['compilation'] = {
    input: hasInput ? { ...compilationOptions.input } : {},
    output,
    ...(minify && { minify }),
    ...(sourcemap && { sourcemap }),
    ...(treeShaking && { treeShaking })
  };

  const defaultOptions: any = {
    compilation,
    watch: !!watch,
    root: options.root,
    server: options.server,
    preview: options.preview,
    clearScreen: !!options.clearScreen,
    configFile: options.configFile,
    timeUnit: options.timeUnit,
    ...(mode && { mode })
  };

  return defaultOptions;
}
