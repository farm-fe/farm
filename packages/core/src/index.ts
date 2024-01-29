export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';

import path from 'node:path';
import fs from 'node:fs/promises';
import { statSync } from 'node:fs';
import fse from 'fs-extra';

import { Compiler } from './compiler/index.js';
import {
  normalizePublicDir,
  resolveConfig,
  UserConfig
} from './config/index.js';
import { DefaultLogger, Logger } from './utils/logger.js';
import { DevServer } from './server/index.js';
import { FileWatcher } from './watcher/index.js';
import { compilerHandler } from './utils/build.js';
import { setProcessEnv } from './config/env.js';
import { colors } from './utils/color.js';

import type {
  FarmCLIOptions,
  ResolvedUserConfig,
  UserPreviewServerConfig
} from './config/types.js';
import { JsPlugin } from './plugin/type.js';
import { __FARM_GLOBAL__ } from './config/_global.js';
import { ConfigWatcher } from './watcher/config-watcher.js';
import { clearScreen } from './utils/share.js';

export async function start(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('development');

  try {
    const resolvedUserConfig = await resolveConfig(
      inlineConfig,
      logger,
      'development'
    );

    const compiler = await createCompiler(resolvedUserConfig);

    const devServer = await createDevServer(
      compiler,
      resolvedUserConfig,
      logger
    );
    await devServer.listen();

    createFileWatcher(devServer, resolvedUserConfig, inlineConfig, logger);

    resolvedUserConfig.jsPlugins.forEach((plugin: JsPlugin) =>
      plugin.configureDevServer?.(devServer)
    );
  } catch (error) {
    logger.error(`Failed to start the server: ${error}`);
  }
}

export async function build(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('production');
  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'production'
  );

  setProcessEnv(resolvedUserConfig.compilation.mode);

  try {
    await createBundleHandler(resolvedUserConfig);
    // copy resources under publicDir to output.path
    await copyPublicDirectory(resolvedUserConfig, inlineConfig, logger);
  } catch (err) {
    console.log(err);
  }
}

export async function preview(inlineConfig: FarmCLIOptions): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  const port = inlineConfig.port ?? 1911;
  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'production'
  );

  const { root, output } = resolvedUserConfig.compilation;
  const distDir = path.resolve(root, output.path);
  try {
    statSync(distDir);
  } catch (err) {
    if (err.code === 'ENOENT') {
      throw new Error(
        `The directory "${distDir}" does not exist. Did you build your project?`
      );
    }
  }

  const previewOptions: UserPreviewServerConfig = {
    distDir,
    output: { path: output.path, publicPath: output.publicPath },
    port,
    host: inlineConfig.host ?? true
  };
  const devServer = new DevServer({ logger });
  devServer.createPreviewServer(previewOptions);
}

export async function watch(
  inlineConfig: FarmCLIOptions & UserConfig
): Promise<void> {
  const logger = inlineConfig.logger ?? new DefaultLogger();
  setProcessEnv('development');

  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'development'
  );

  setProcessEnv(resolvedUserConfig.compilation.mode);

  const compilerFileWatcher = await createBundleHandler(
    resolvedUserConfig,
    true
  );

  async function handleFileChange(files: string[]) {
    logFileChanges(files, resolvedUserConfig.root, logger);

    try {
      farmWatcher.close();

      __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;

      compilerFileWatcher?.close();

      await watch(inlineConfig);
    } catch (error) {
      logger.error(`Error restarting the watcher: ${error.message}`);
    }
  }

  const farmWatcher = new ConfigWatcher(resolvedUserConfig).watch(
    handleFileChange
  );
}

export async function clean(
  rootPath: string,
  recursive: boolean | undefined
): Promise<void> {
  // TODO After optimizing the reading of config, put the clean method into compiler
  const logger = new DefaultLogger();

  const nodeModulesFolders = recursive
    ? await findNodeModulesRecursively(rootPath)
    : [path.join(rootPath, 'node_modules')];

  await Promise.all(
    nodeModulesFolders.map(async (nodeModulesPath) => {
      // TODO Bug .farm cacheDir folder not right
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

async function findNodeModulesRecursively(rootPath: string): Promise<string[]> {
  const result: string[] = [];

  async function traverse(currentPath: string) {
    const items = await fs.readdir(currentPath);
    for (const item of items) {
      const fullPath = path.join(currentPath, item);
      const stats = await fs.stat(fullPath);

      if (stats.isDirectory()) {
        if (item === 'node_modules') {
          result.push(fullPath);
        } else {
          await traverse(fullPath);
        }
      }
    }
  }

  await traverse(rootPath);
  return result;
}

export async function createBundleHandler(
  resolvedUserConfig: ResolvedUserConfig,
  watchMode = false
) {
  const compiler = await createCompiler(resolvedUserConfig);

  await compilerHandler(async () => {
    compiler.removeOutputPathDir();
    await compiler.compile();
    compiler.writeResourcesToDisk();
  }, resolvedUserConfig);

  if (resolvedUserConfig.compilation?.watch || watchMode) {
    const watcher = new FileWatcher(compiler, resolvedUserConfig);
    await watcher.watch();
    return watcher;
  }
}

export async function createCompiler(resolvedUserConfig: ResolvedUserConfig) {
  const {
    jsPlugins,
    rustPlugins,
    compilation: compilationConfig
  } = resolvedUserConfig;

  const compiler = new Compiler({
    config: compilationConfig,
    jsPlugins,
    rustPlugins
  });

  for (const plugin of jsPlugins) {
    await plugin.configureCompiler?.(compiler);
  }

  return compiler;
}

async function copyPublicDirectory(
  resolvedUserConfig: ResolvedUserConfig,
  inlineConfig: FarmCLIOptions & UserConfig,
  logger: Logger
): Promise<void> {
  const absPublicDirPath = normalizePublicDir(
    resolvedUserConfig.root,
    inlineConfig.publicDir
  );

  try {
    if (await fse.pathExists(absPublicDirPath)) {
      await fse.copy(
        absPublicDirPath,
        resolvedUserConfig.compilation.output.path
      );
      logger.info(
        `Public directory resources copied ${colors.bold(
          colors.green('successfully')
        )}.`
      );
    }
  } catch (error) {
    logger.error(`Error copying public directory: ${error.message}`);
  }
}

export async function createDevServer(
  compiler: Compiler,
  resolvedUserConfig: ResolvedUserConfig,
  logger: Logger
) {
  const devServer = new DevServer({ compiler, logger });
  await devServer.createDevServer(resolvedUserConfig.server);

  return devServer;
}

export async function createFileWatcher(
  devServer: DevServer,
  resolvedUserConfig: ResolvedUserConfig,
  inlineConfig: FarmCLIOptions & UserConfig,
  logger: Logger
) {
  if (
    devServer.config.hmr &&
    resolvedUserConfig.compilation.mode === 'production'
  ) {
    logger.error('HMR cannot be enabled in production mode.');
    return;
  }

  if (!devServer.config.hmr) {
    return;
  }

  const fileWatcher = new FileWatcher(devServer, resolvedUserConfig);
  devServer.watcher = fileWatcher;
  await fileWatcher.watch();

  const farmWatcher = new ConfigWatcher(resolvedUserConfig).watch(
    async (files: string[]) => {
      clearScreen();
      logFileChanges(files, resolvedUserConfig.root, logger);

      devServer.restart(async () => {
        farmWatcher?.close();
        await devServer.close();
        __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
        await start(inlineConfig);
      });
    }
  );
}

export function logFileChanges(files: string[], root: string, logger: Logger) {
  const changedFiles = files
    .map((file) => path.relative(root, file))
    .join(', ');
  logger.info(
    colors.bold(colors.green(`${changedFiles} changed, server will restart.`))
  );
}

export { defineFarmConfig as defineConfig } from './config/index.js';
