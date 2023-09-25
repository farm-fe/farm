import { basename, relative } from 'node:path';
import { createRequire } from 'node:module';
import debounce from 'lodash.debounce';
import chalk from 'chalk';

import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { Config, JsFileWatcher } from '../../binding/index.js';
import { compilerHandler, DefaultLogger, clearScreen } from '../utils/index.js';
import {
  DEFAULT_HMR_OPTIONS,
  JsPlugin,
  normalizeUserCompilationConfig,
  resolveUserConfig
} from '../index.js';
import { __FARM_GLOBAL__ } from '../config/_global.js';

import type { UserConfig } from '../config/index.js';
import { setProcessEnv } from '../config/env.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: JsFileWatcher;
  private _logger: DefaultLogger;
  private _awaitWriteFinish: number;

  constructor(
    public serverOrCompiler: DevServer | Compiler,
    public options?: Config & UserConfig
  ) {
    this._root = options.config.root;
    this._awaitWriteFinish = DEFAULT_HMR_OPTIONS.watchOptions.awaitWriteFinish;

    if (serverOrCompiler instanceof DevServer) {
      this._awaitWriteFinish =
        serverOrCompiler.config.hmr.watchOptions.awaitWriteFinish ??
        this._awaitWriteFinish;
    } else if (serverOrCompiler instanceof Compiler) {
      this._awaitWriteFinish =
        serverOrCompiler.config.config?.watch?.watchOptions?.awaitWriteFinish ??
        this._awaitWriteFinish;
    }

    this._logger = new DefaultLogger();
  }

  async watch() {
    // Determine how to compile the project
    const compiler = this.getCompilerFromServerOrCompiler(
      this.serverOrCompiler
    );

    let handlePathChange = async (path: string): Promise<void> => {
      const fileName = basename(path);
      const isEnv = fileName === '.env' || fileName.startsWith('.env.');
      const isConfig = path === this.options.resolveConfigPath;

      // TODO configFileDependencies e.g: isDependencies = ["./farm.config.ts"]
      if (isEnv || isConfig) {
        clearScreen();
        __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = false;
        this._logger.info(
          `restarting server due to ${chalk.green(
            relative(process.cwd(), path)
          )} change`
        );
        if (this.serverOrCompiler instanceof DevServer) {
          await this.serverOrCompiler.close();
        }
        const config: UserConfig = await resolveUserConfig(
          this.options.inlineConfig,
          this._logger
        );
        const normalizedConfig = await normalizeUserCompilationConfig(config);
        setProcessEnv(normalizedConfig.config.mode);
        const compiler = new Compiler(normalizedConfig);
        const devServer = new DevServer(compiler, this._logger, config);
        this.serverOrCompiler = devServer;
        await devServer.listen();
        if (normalizedConfig.config.mode === 'development') {
          normalizedConfig.jsPlugins.forEach((plugin: JsPlugin) =>
            plugin.configDevServer?.(devServer)
          );
        }
        return;
      }
      try {
        if (this.serverOrCompiler instanceof DevServer) {
          await this.serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (
          this.serverOrCompiler instanceof Compiler &&
          this.serverOrCompiler.hasModule(path)
        ) {
          compilerHandler(async () => {
            await compiler.update([path], true);
            compiler.writeResourcesToDisk();
          }, this.options);
        }
      } catch (error) {
        this._logger.error(error);
      }
    };

    if (process.platform === 'win32') {
      handlePathChange = debounce(handlePathChange, this._awaitWriteFinish);
    }

    this._watcher = new JsFileWatcher((paths: string[]) => {
      paths.forEach(handlePathChange);
    });

    this._watcher.watch([
      ...compiler.resolvedModulePaths(this._root),
      ...compiler.resolvedWatchPaths()
    ]);

    if (this.serverOrCompiler instanceof DevServer) {
      this.serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
        const added = [
          ...updateResult.added,
          ...updateResult.extraWatchResult.add
        ].map((addedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            addedModule
          );
          return resolvedPath;
        });
        this._watcher.watch(added);
      });
    }
  }

  private getCompilerFromServerOrCompiler(
    serverOrCompiler: DevServer | Compiler
  ): Compiler {
    return serverOrCompiler instanceof DevServer
      ? serverOrCompiler.getCompiler()
      : serverOrCompiler;
  }
}

export async function restartServer(server: DevServer) {
  await server.close();
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function clearModuleCache(modulePath: string) {
  const _require = createRequire(import.meta.url);
  delete _require.cache[_require.resolve(modulePath)];
}
