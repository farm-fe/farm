export * from './compiler/index.js';
export * from './config/index.js';
export * from './server/index.js';
export * from './plugin/type.js';
export * from './utils/index.js';

import { statSync } from 'node:fs';
import fs from 'node:fs/promises';
import path from 'node:path';
import fse from 'fs-extra';

import { Compiler } from './compiler/index.js';
import { loadEnv, setProcessEnv } from './config/env.js';
import {
  UserConfig,
  getConfigFilePath,
  normalizePublicDir,
  resolveConfig
} from './config/index.js';
import { Server } from './server/index.js';
import { compilerHandler } from './utils/build.js';
import { colors } from './utils/color.js';
import { Logger } from './utils/logger.js';
import { FileWatcher } from './watcher/index.js';

import { __FARM_GLOBAL__ } from './config/_global.js';
import type {
  FarmCLIOptions,
  ResolvedUserConfig,
  UserPreviewServerConfig
} from './config/types.js';
import { logError } from './server/error.js';
import { lazyCompilation } from './server/middlewares/lazy-compilation.js';
import { resolveHostname } from './utils/http.js';
import { clearScreen } from './utils/share.js';
import { ConfigWatcher } from './watcher/config-watcher.js';

import type { JsPlugin } from './plugin/type.js';

export async function start(
  inlineConfig?: FarmCLIOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  const logger = inlineConfig.logger ?? new Logger();
  setProcessEnv('development');

  try {
    const resolvedUserConfig = await resolveConfig(
      inlineConfig,
      logger,
      'development'
    );

    const compiler = await createCompiler(resolvedUserConfig, logger);

    const devServer = await createDevServer(
      compiler,
      resolvedUserConfig,
      logger
    );

    const watcher = await createFileWatcher(
      devServer,
      resolvedUserConfig,
      inlineConfig,
      logger
    );
    // call configureDevServer hook after both server and watcher are ready
    resolvedUserConfig.jsPlugins.forEach((plugin: JsPlugin) =>
      plugin.configureDevServer?.(devServer)
    );

    await devServer.listen();
    watcher.watchExtraFiles();
  } catch (error) {
    logger.error(`Failed to start the server: \n ${error}`, { exit: true });
  }
}

export async function build(
  inlineConfig?: FarmCLIOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  const logger = inlineConfig.logger ?? new Logger();
  setProcessEnv('production');

  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'production',
    false
  );

  try {
    await createBundleHandler(resolvedUserConfig, logger);
    // copy resources under publicDir to output.path
    await copyPublicDirectory(resolvedUserConfig, logger);
  } catch (err) {
    logger.error(`Failed to build: ${err}`, { exit: true });
  }
}

export async function preview(inlineConfig?: FarmCLIOptions): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  const logger = inlineConfig.logger ?? new Logger();
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

  // reusing port conflict check from DevServer
  const serverConfig = {
    ...resolvedUserConfig.server,
    host: inlineConfig.host ?? true,
    port:
      inlineConfig.port ??
      (Number(process.env.FARM_DEFAULT_SERVER_PORT) || 1911)
  };
  await Server.resolvePortConflict(serverConfig, logger);
  const port = serverConfig.port;
  const host = serverConfig.host;
  const previewOptions: UserPreviewServerConfig = {
    ...serverConfig,
    distDir,
    output: { path: output.path, publicPath: output.publicPath },
    port,
    host
  };

  const server = new Server({ logger });
  server.createPreviewServer(previewOptions);
}

export async function watch(
  inlineConfig?: FarmCLIOptions & UserConfig
): Promise<void> {
  inlineConfig = inlineConfig ?? {};
  const logger = inlineConfig.logger ?? new Logger();
  setProcessEnv('development');

  const resolvedUserConfig = await resolveConfig(
    inlineConfig,
    logger,
    'development',
    true
  );

  const hostname = await resolveHostname(resolvedUserConfig.server.host);
  resolvedUserConfig.compilation.define = {
    ...(resolvedUserConfig.compilation.define ?? {}),
    FARM_NODE_LAZY_COMPILE_SERVER_URL: `http://${
      hostname.host || 'localhost'
    }:${resolvedUserConfig.server.port}`
  };

  const compilerFileWatcher = await createBundleHandler(
    resolvedUserConfig,
    logger,
    true
  );

  const lazyEnabled = resolvedUserConfig.compilation?.lazyCompilation;
  let devServer: Server | undefined;
  // create dev server for lazy compilation
  if (lazyEnabled) {
    devServer = new Server({
      logger,
      compiler: compilerFileWatcher.serverOrCompiler as Compiler
    });
    await devServer.createServer(resolvedUserConfig.server);
    devServer.applyMiddlewares([lazyCompilation]);
    await devServer.startServer(resolvedUserConfig.server);
  }

  async function handleFileChange(files: string[]) {
    logFileChanges(files, resolvedUserConfig.root, logger);

    try {
      farmWatcher.close();

      if (lazyEnabled && devServer) {
        devServer.close();
      }

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
  recursive?: boolean | undefined
): Promise<void> {
  // TODO After optimizing the reading of config, put the clean method into compiler
  const logger = new Logger();

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
  logger: Logger,
  watchMode = false
) {
  const compiler = await createCompiler(resolvedUserConfig, logger);

  await compilerHandler(
    async () => {
      compiler.removeOutputPathDir();
      try {
        await compiler.compile();
      } catch (err) {
        throw new Error(logError(err) as unknown as string);
      }
      compiler.writeResourcesToDisk();
    },
    resolvedUserConfig,
    logger
  );

  if (resolvedUserConfig.compilation?.watch || watchMode) {
    const watcher = new FileWatcher(compiler, resolvedUserConfig, logger);
    await watcher.watch();
    return watcher;
  }
}

export async function createCompiler(
  resolvedUserConfig: ResolvedUserConfig,
  logger: Logger
) {
  const {
    jsPlugins,
    rustPlugins,
    compilation: compilationConfig
  } = resolvedUserConfig;

  const compiler = new Compiler(
    {
      config: compilationConfig,
      jsPlugins,
      rustPlugins
    },
    logger
  );

  for (const plugin of jsPlugins) {
    await plugin.configureCompiler?.(compiler);
  }

  return compiler;
}

async function copyPublicDirectory(
  resolvedUserConfig: ResolvedUserConfig,
  logger: Logger
): Promise<void> {
  const absPublicDirPath = normalizePublicDir(
    resolvedUserConfig.root,
    resolvedUserConfig.publicDir
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
  const server = new Server({ compiler, logger });
  await server.createDevServer(resolvedUserConfig.server);

  return server;
}

export async function createFileWatcher(
  devServer: Server,
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

  const fileWatcher = new FileWatcher(devServer, resolvedUserConfig, logger);
  devServer.watcher = fileWatcher;
  await fileWatcher.watch();

  // const farmWatcher = new ConfigWatcher(resolvedUserConfig);
  const configFilePath = await getConfigFilePath(
    inlineConfig.configPath ?? resolvedUserConfig.root
  );
  const farmWatcher = new ConfigWatcher({
    ...resolvedUserConfig,
    configFilePath
  });
  farmWatcher.watch(async (files: string[]) => {
    clearScreen();

    devServer.restart(async () => {
      logFileChanges(files, resolvedUserConfig.root, logger);
      farmWatcher?.close();

      await devServer.close();
      __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
      await start(inlineConfig);
    });
  });
  return fileWatcher;
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

export { loadEnv };
