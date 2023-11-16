import { createRequire } from 'node:module';

import { Config } from '../../../binding/index.js';
import { ResolvedUserConfig } from '../index.js';
import { RustPlugin } from '../../plugin/index.js';
import { traceDependenciesHash } from '../../utils/trace-dependencies.js';
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
      buildDependencies: []
    };
  }

  // trace all build dependencies of the config file
  if (config.configFilePath) {
    config.persistentCache.buildDependencies = [
      await traceDependenciesHash(config.configFilePath)
    ];

    const rustPlugins = resolvedUserConfig.plugins?.filter(
      (plugin) => typeof plugin === 'string' || Array.isArray(plugin)
    ) as RustPlugin[];

    if (rustPlugins?.length) {
      const require = createRequire(path.join(config.root, 'package.json'));

      for (const p of rustPlugins) {
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
          continue;
        }
      }
    }
  }
}
