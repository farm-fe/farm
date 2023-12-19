import { createRequire } from 'node:module';

import { Config } from '../../../binding/index.js';
import { ResolvedUserConfig } from '../index.js';
import { RustPlugin } from '../../plugin/index.js';
import { traceDependencies } from '../../utils/trace-dependencies.js';
import path from 'node:path';

export async function normalizePersistentCache(
  config: Config['config'],
  resolvedUserConfig: ResolvedUserConfig
) {
  if (
    config?.persistentCache === false ||
    config.configFilePath === undefined
  ) {
    return;
  }

  if (config.persistentCache === true || config.persistentCache == undefined) {
    config.persistentCache = {
      buildDependencies: [],
      moduleCacheKeyStrategy: {},
      envs: config.env
    };
  }

  if (config.persistentCache.envs === undefined) {
    config.persistentCache.envs = config.env;
  }

  if (!config.persistentCache.buildDependencies) {
    config.persistentCache.buildDependencies = [];
  }

  for (const lockfile of ['package-lock.json', 'yarn.lock', 'pnpm-lock.yaml']) {
    if (!config.persistentCache.buildDependencies.includes(lockfile)) {
      config.persistentCache.buildDependencies.push(lockfile);
    }
  }

  if (config?.output?.targetEnv === 'node') {
    if (!config.persistentCache.moduleCacheKeyStrategy) {
      config.persistentCache.moduleCacheKeyStrategy = {};
    }

    config.persistentCache.moduleCacheKeyStrategy.timestamp = false;
  }

  // trace all build dependencies of the config file
  if (config.configFilePath) {
    const files = resolvedUserConfig?.configFileDependencies?.length
      ? resolvedUserConfig.configFileDependencies
      : await traceDependencies(config.configFilePath);

    const packages = [];

    for (const file of files) {
      if (path.isAbsolute(file)) {
        config.persistentCache.buildDependencies.push(file);
      } else {
        packages.push(file);
      }
    }

    const rustPlugins = resolvedUserConfig.plugins?.filter(
      (plugin) => typeof plugin === 'string' || Array.isArray(plugin)
    ) as RustPlugin[];

    packages.push(...(rustPlugins ?? []));

    if (packages?.length) {
      const require = createRequire(path.join(config.root, 'package.json'));

      for (const p of packages) {
        try {
          let packageJsonPath: string;
          if (typeof p === 'string') {
            packageJsonPath = require.resolve(`${p}/package.json`);
          } else {
            packageJsonPath = require.resolve(`${p[0]}/package.json`);
          }

          const packageJson = require(packageJsonPath);
          const key = `${packageJson.name}@${packageJson.version}`;
          config.persistentCache.buildDependencies.push(key);
        } catch (e) {
          if (typeof p === 'string') {
            config.persistentCache.buildDependencies.push(p);
          } else if (Array.isArray(p) && typeof p[0] === 'string') {
            config.persistentCache.buildDependencies.push(p[0]);
          }
          continue;
        }
      }
    }

    config.persistentCache.buildDependencies.sort();
  }
}
