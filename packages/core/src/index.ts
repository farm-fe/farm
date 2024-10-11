export * from './compiler/index';
export * from './config/index';
export * from './server/index';
export * from './plugin/type';
export * from './utils/index';

export { defineFarmConfig as defineConfig } from './config/index';

export type { Compiler as BindingCompiler } from './types/binding';

import fs from 'node:fs/promises';

import { createCompiler } from './compiler/index';
import { __FARM_GLOBAL__ } from './config/_global';
import { UserConfig, resolveConfig } from './config/index';
import { getPluginHooks } from './plugin/index';
import { Server } from './server/index';
import { PersistentCacheConfig } from './types/binding';
import { PersistentCacheBrand, bold, colors, green } from './utils/color';
import {
  copyPublicDirectory,
  findNodeModulesRecursively
} from './utils/fsUtils.js';
import { getShortName } from './utils/path';
import { handlerWatcher } from './watcher/index';

import type { FarmCliOptions } from './config/types';

export async function start(
  inlineConfig?: FarmCliOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  const server = new Server(inlineConfig);
  try {
    await server.createServer();
    server.listen();
  } catch (error) {
    server.logger.error('Failed to start the server', { exit: false, error });
  }
}

export async function build(
  inlineConfig: FarmCliOptions & UserConfig = {}
): Promise<void> {
  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    'build',
    'production',
    'production'
  );

  const { persistentCache, output } = resolvedUserConfig.compilation;

  try {
    const compiler = await createCompiler(resolvedUserConfig);

    for (const hook of getPluginHooks(
      resolvedUserConfig.jsPlugins,
      'configureCompiler'
    )) {
      await hook?.(compiler);
    }

    if (output?.clean) {
      compiler.removeOutputPathDir();
    }

    const startTime = performance.now();
    await compiler.compile();
    const elapsedTime = Math.floor(performance.now() - startTime);
    const persistentCacheText = persistentCache
      ? bold(PersistentCacheBrand)
      : '';

    const shortFile = getShortName(
      resolvedUserConfig.configFilePath,
      resolvedUserConfig.root
    );
    resolvedUserConfig.logger.info(
      `Using config file at ${bold(green(shortFile))}`
    );
    resolvedUserConfig.logger.info(
      `Build completed in ${bold(
        green(`${elapsedTime}ms`)
      )} ${persistentCacheText} Resources emitted to ${bold(
        green(output.path)
      )}.`
    );
    compiler.writeResourcesToDisk();
    await copyPublicDirectory(resolvedUserConfig);
    if (resolvedUserConfig.watch) {
      handlerWatcher(resolvedUserConfig, compiler);
    }
  } catch (err) {
    resolvedUserConfig.logger.error(`Failed to build: ${err}`, { exit: true });
  }
}

// TODO preview method
export async function preview(inlineConfig: FarmCliOptions & UserConfig = {}) {}

export async function clean(
  rootPath: string,
  recursive?: boolean | undefined
): Promise<void> {
  const resolvedUserConfig = await resolveConfig(
    {},
    'build',
    'production',
    'production'
  );
  const cachePath = (
    resolvedUserConfig.compilation.persistentCache as PersistentCacheConfig
  ).cacheDir;

  const nodeModulesFolders = recursive
    ? await findNodeModulesRecursively(rootPath)
    : [cachePath];

  await Promise.all(
    nodeModulesFolders.map(async (nodeModulesPath) => {
      try {
        const farmFolderStats = await fs.stat(cachePath);
        if (farmFolderStats.isDirectory()) {
          await fs.rm(cachePath, { recursive: true, force: true });
          resolvedUserConfig.logger.info(
            `✨ ✨ Cache cleaned at ${colors.bold(colors.green(cachePath))}`
          );
        }
      } catch (error) {
        if (error?.code === 'ENOENT') {
          resolvedUserConfig.logger.warn(
            `No cached files found in ${colors.bold(
              colors.green(nodeModulesPath)
            )}`
          );
        } else {
          resolvedUserConfig.logger.error(
            `Error cleaning cache in ${colors.bold(
              colors.green(nodeModulesPath)
            )}: ${error.message}`
          );
        }
      }
    })
  );
}
