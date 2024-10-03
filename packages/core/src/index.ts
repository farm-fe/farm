export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';

export { defineFarmConfig as defineConfig } from './config/index.js';

export type {
  ModuleType,
  ResolveKind,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  OutputConfig,
  ResolveConfig,
  RuntimeConfig,
  ScriptConfig,
  CssConfig,
  PersistentCacheConfig,
  PartialBundlingConfig,
  PresetEnvConfig,
  Config,
  PluginTransformHookParam,
  PluginTransformHookResult
} from './types/binding.js';

import fs from 'node:fs/promises';
import path, { resolve } from 'node:path';

import { UserConfig, resolveConfig } from './config/index.js';
import { Server } from './server/index.js';
import { PersistentCacheBrand, bold, colors, green } from './utils/color.js';
import { Logger } from './utils/logger.js';

import { JsUpdateResult } from '../binding/binding.js';
import {
  createCompiler,
  resolveConfigureCompilerHook
} from './compiler/utils.js';
import { __FARM_GLOBAL__ } from './config/_global.js';
import type { FarmCliOptions, ResolvedUserConfig } from './config/types.js';
import { convertErrorMessage } from './utils/error.js';
import {
  copyPublicDirectory,
  findNodeModulesRecursively
} from './utils/fsUtils.js';
import { getShortName } from './utils/path.js';
import { normalizePath } from './utils/share.js';
import Watcher from './watcher/index.js';

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
  inlineConfig?: FarmCliOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};

  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    'build',
    'production',
    'production'
  );

  const { persistentCache, output } = resolvedUserConfig.compilation;

  try {
    const compiler = await createCompiler(resolvedUserConfig);
    await resolveConfigureCompilerHook(compiler, resolvedUserConfig);

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
      )} ${persistentCacheText} Resources emitted to ${bold(green(output.path))}.`
    );
    compiler.writeResourcesToDisk();
    await copyPublicDirectory(resolvedUserConfig);
    if (resolvedUserConfig.watch) {
      // TODO 传 一个 参数 多加一个 compiler 的参数
      const watcher = new Watcher(resolvedUserConfig);

      await watcher.createWatcher();
      watcher.watcher.on('change', async (file: string | string[] | any) => {
        file = normalizePath(file);
        const shortFile = getShortName(file, resolvedUserConfig.root);
        const isConfigFile = resolvedUserConfig.configFilePath === file;
        const isConfigDependencyFile =
          resolvedUserConfig.configFileDependencies.some(
            (name) => file === name
          );
        const isEnvFile = resolvedUserConfig.envFiles.some(
          (name) => file === name
        );
        if (isConfigFile || isConfigDependencyFile || isEnvFile) {
          __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
          resolvedUserConfig.logger.info(
            `${bold(green(shortFile))} changed, Bundler Config is being reloaded`,
            true
          );
          try {
            // TODO 是否需要做这种操作
            console.log('restart build config compiler');
            return;
          } catch (e) {
            resolvedUserConfig.logger.error(`restart server error ${e}`);
          }
        }
        const handleUpdateFinish = (updateResult: JsUpdateResult) => {
          const added = [
            ...updateResult.added,
            ...updateResult.extraWatchResult.add
          ].map((addedModule) => {
            const resolvedPath = compiler.transformModulePath(
              resolvedUserConfig.root,
              addedModule
            );
            return resolvedPath;
          });

          const filteredAdded = added.filter((file) =>
            watcher.filterWatchFile(file, resolvedUserConfig.root)
          );

          if (filteredAdded.length > 0) {
            watcher.watcher.add(filteredAdded);
          }
        };

        try {
          const start = performance.now();
          const result = await compiler.update([file], true);
          const elapsedTime = Math.floor(performance.now() - start);
          resolvedUserConfig.logger.info(
            `Build completed in ${bold(
              green(`${elapsedTime}ms`)
            )} ${persistentCacheText} Resources emitted to ${bold(green(output.path))}.`
          );
          handleUpdateFinish(result);
          compiler.writeResourcesToDisk();
        } catch (error) {
          resolvedUserConfig.logger.error(
            `Farm Update Error: ${convertErrorMessage(error)}`
          );
        }
      });
    }
  } catch (err) {
    resolvedUserConfig.logger.error(`Failed to build: ${err}`, { exit: true });
  }
}

export async function clean(
  rootPath: string,
  recursive?: boolean | undefined
): Promise<void> {
  // TODO After optimizing the reading of config, put the clean method into compiler
  const logger = new Logger();

  const nodeModulesFolders = recursive
    ? await findNodeModulesRecursively(rootPath)
    : [path.join(rootPath, 'node_modules')];

  await Promise.all(
    nodeModulesFolders.map(async (nodeModulesPath) => {
      // TODO Bug .farm cacheDir folder not right find config
      const farmFolderPath = path.join(nodeModulesPath, '.farm');
      try {
        const stats = await fs.stat(farmFolderPath);
        if (stats.isDirectory()) {
          await fs.rm(farmFolderPath, { recursive: true, force: true });
          // TODO optimize nodeModulePath path e.g: /Users/xxx/node_modules/.farm/cache
          logger.info(
            `Cache cleaned at ${colors.bold(colors.green(nodeModulesPath))}`
          );
        }
      } catch (error) {
        if (error.code === 'ENOENT') {
          logger.warn(
            `No cached files found in ${colors.bold(
              colors.green(nodeModulesPath)
            )}`
          );
        } else {
          logger.error(
            `Error cleaning cache in ${colors.bold(
              colors.green(nodeModulesPath)
            )}: ${error.message}`
          );
        }
      }
    })
  );
}
