export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './plugin/index.js';
export * from './utils/index.js';

export { defineFarmConfig as defineConfig } from './config/index.js';

import fs from 'node:fs/promises';

import { createCompiler } from './compiler/index.js';
import { __FARM_GLOBAL__ } from './config/_global.js';
import { UserConfig, resolveConfig } from './config/index.js';
import { getPluginHooks } from './plugin/index.js';
import { PreviewServer, Server } from './server/index.js';
import {
  PersistentCacheBrand,
  bold,
  colors,
  findNodeModulesRecursively,
  getShortName,
  green
} from './utils/index.js';
import { handlerWatcher } from './watcher/index.js';

import type { FarmCliOptions } from './config/types.js';
export type { Compiler as BindingCompiler } from './types/binding.js';
import type { PersistentCacheConfig } from './types/binding.js';

export async function start(
  inlineConfig?: FarmCliOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  const server = new Server(inlineConfig);
  try {
    await server.createServer();
    await server.listen();
    server.printUrls();
  } catch (error) {
    server.logger.error('Failed to start the server', { exit: false, error });
  }
}

export async function preview(
  inlineConfig: FarmCliOptions & UserConfig = {}
): Promise<void> {
  const previewServer = new PreviewServer(inlineConfig);
  try {
    await previewServer.createPreviewServer();
    previewServer.listen();
  } catch (error) {
    previewServer.logger.error('Failed to start the preview server', {
      exit: false,
      error
    });
  }
}

export async function build(
  inlineConfig: FarmCliOptions & UserConfig = {}
): Promise<void> {
  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    'build',
    'production'
  );
  const {
    compilation: { persistentCache, output },
    configFilePath,
    root,
    logger,
    jsPlugins,
    watch
  } = resolvedUserConfig;

  try {
    const shortFile = getShortName(configFilePath, root);
    logger.info(`Using config file at ${bold(green(shortFile))}`);

    const compiler = createCompiler(resolvedUserConfig);
    for (const hook of getPluginHooks(jsPlugins, 'configureCompiler')) {
      await hook?.(compiler);
    }

    // TODO move to rust
    if (output?.clean) {
      compiler.removeOutputPathDir();
    }
    const startTime = performance.now();
    await compiler.compile();
    const elapsedTime = Math.floor(performance.now() - startTime);
    const persistentCacheText = persistentCache
      ? bold(PersistentCacheBrand)
      : '';

    logger.info(
      `Build completed in ${bold(
        green(`${logger.formatTime(elapsedTime)}`)
      )} ${persistentCacheText} Resources emitted to ${bold(
        green(output.path)
      )}.`
    );
    compiler.writeResourcesToDisk();
    if (watch) {
      handlerWatcher(resolvedUserConfig, compiler);
    }
  } catch (err) {
    logger.error(`Failed to build: ${err}`, { exit: true });
  }
}

export async function clean(
  rootPath: string,
  recursive = false
): Promise<void> {
  const resolvedUserConfig = await resolveConfig({}, 'build', 'production');
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
