import { createRequire } from 'node:module';
import path from 'node:path';
import { existsSync, readFileSync } from 'node:fs';

import { Config } from '../../../binding/index.js';
import { ResolvedUserConfig } from '../index.js';
import { RustPlugin } from '../../plugin/index.js';
import { traceDependencies } from '../../utils/trace-dependencies.js';

export async function normalizePersistentCache(
  config: Config['config'],
  resolvedUserConfig: ResolvedUserConfig
) {
  if (
    config?.persistentCache === false ||
    resolvedUserConfig.configFilePath === undefined
  ) {
    return;
  }

  if (config.persistentCache === true || config.persistentCache == undefined) {
    config.persistentCache = {
      buildDependencies: [],
      moduleCacheKeyStrategy: {},
      envs: resolvedUserConfig.env
    };
  }

  if (config.persistentCache.envs === undefined) {
    config.persistentCache.envs = resolvedUserConfig.env;
  } else if (typeof config.persistentCache.envs === 'object') {
    config.persistentCache.envs = {
      ...resolvedUserConfig.env,
      ...config.persistentCache.envs
    };
  }

  // all define options should be in envs
  if (config.define && typeof config.define === 'object') {
    config.persistentCache.envs = {
      ...config.persistentCache.envs,
      ...Object.entries(config.define)
        .map(([k, v]) =>
          typeof v !== 'string' ? [k, JSON.stringify(v)] : [k, v]
        )
        .reduce((acc, [k, v]) => {
          acc[k] = v;
          return acc;
        }, {} as Record<string, string>)
    };
  }

  // add type of package.json to envs
  const packageJsonPath = path.join(
    config.root ?? process.cwd(),
    'package.json'
  );

  if (existsSync(packageJsonPath)) {
    const s = readFileSync(packageJsonPath).toString();
    const packageJson = JSON.parse(s);
    const affectedKeys = [
      'type',
      'name',
      'exports',
      'browser',
      'main',
      'module'
    ];

    for (const key of affectedKeys) {
      const value = packageJson[key] ?? 'unknown';
      config.persistentCache.envs[`package.json[${key}]`] =
        typeof value !== 'string' ? JSON.stringify(value) : value;
    }
  }

  if (!config.persistentCache.buildDependencies) {
    config.persistentCache.buildDependencies = [];
  }
  // TODO find latest lock file starting from root
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
  if (resolvedUserConfig.configFilePath) {
    const files = resolvedUserConfig?.configFileDependencies?.length
      ? resolvedUserConfig.configFileDependencies
      : await traceDependencies(resolvedUserConfig.configFilePath);

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
      // console.log('packages', config);
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
