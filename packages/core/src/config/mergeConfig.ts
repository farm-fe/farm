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
      'plugins',
      'publicDir',
      'server',
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
      const resolvedRoot = path.resolve(cliOption.configPath, target.root);
      target.root = resolvedRoot;
    }
  }

  if (isString(options.host) || typeof options.host === 'boolean') {
    left = mergeConfig(left, { server: { host: options.host } });
  }

  if (typeof options.minify === 'boolean') {
    left = mergeConfig(left, { compilation: { minify: options.minify } });
  }

  if (options.outDir) {
    left = mergeConfig(left, {
      compilation: { output: { path: options.outDir } }
    });
  }

  if (options.port) {
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

  if (options.https) {
    left = mergeConfig(left, {
      server: {
        https: options.https
      }
    });
  }

  if (options.sourcemap) {
    left = mergeConfig(left, {
      compilation: { sourcemap: options.sourcemap }
    });
  }

  return mergeConfig(left, target);
}

export function initialCliOptions(options: any): any {
  const {
    input,
    outDir,
    target,
    format,
    watch,
    minify,
    sourcemap,
    treeShaking,
    mode
  } = options;

  const output: UserConfig['compilation']['output'] = {
    ...(outDir && { path: outDir }),
    ...(target && { targetEnv: target }),
    ...(format && { format })
  };

  const compilation: UserConfig['compilation'] = {
    input: { ...(input && { index: input }) },
    output,
    ...(watch && { watch }),
    ...(minify && { minify }),
    ...(sourcemap && { sourcemap }),
    ...(treeShaking && { treeShaking })
  };

  const defaultOptions: any = {
    compilation,
    root: options.root,
    configFile: options.configFile,
    ...(mode && { mode })
  };

  return defaultOptions;
}
