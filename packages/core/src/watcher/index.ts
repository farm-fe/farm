import chokidar, { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { DefaultLogger } from '../utils/logger.js';

import { Config } from '../../binding/index.js';
import { isObject } from '../utils/common.js';

import type { WatchOptions as ChokidarFileWatcherOptions } from 'chokidar';
import { compilerHandler } from '../utils/build.js';

export class FileWatcher {
  private _root: string;
  private _watcher: FSWatcher;
  private _options: Config;
  private _logger: DefaultLogger;

  constructor(root: string, options?: Config) {
    this._root = root;
    this._logger = new DefaultLogger();
    this._options = options ?? {};
  }

  async watch(serverOrCompiler: DevServer | Compiler, config: Config) {
    // Determine how to compile the project
    const compiler = this.getCompilerFromServerOrCompiler(serverOrCompiler);

    const watcherOptions = this.resolvedWatcherOptions();

    if (compiler instanceof DevServer) {
      this._watcher = chokidar.watch(compiler.resolvedModulePaths(this._root));
      compiler.hmrEngine?.onUpdateFinish((updateResult) => {
        const added = updateResult.added.map((addedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            addedModule
          );
          return resolvedPath;
        });
        this._watcher.add(added);

        const removed = updateResult.removed.map((removedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            removedModule
          );
          return resolvedPath;
        });

        this._watcher.unwatch(removed);
      });
    }

    if (compiler instanceof Compiler) {
      this._watcher = chokidar.watch(
        compiler.resolvedModulePaths(this._root),
        watcherOptions as ChokidarFileWatcherOptions
      );
    }

    this._watcher.on('change', async (path: string) => {
      try {
        if (serverOrCompiler instanceof DevServer) {
          await serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (serverOrCompiler instanceof Compiler) {
          compilerHandler(async () => {
            await compiler.update([path], true);
            compiler.writeResourcesToDisk();
          }, config);
        }
      } catch (error) {
        this._logger.error(error);
      }
    });
  }

  private getCompilerFromServerOrCompiler(
    serverOrCompiler: DevServer | Compiler
  ): Compiler {
    return serverOrCompiler instanceof DevServer
      ? serverOrCompiler.getCompiler()
      : serverOrCompiler;
  }

  private resolvedWatcherOptions() {
    const watchOptionsType = isObject(this._options.config?.watch);
    const userWatcherOptions = watchOptionsType
      ? this._options.config.watch
      : {};
    const { ignored = [] } = userWatcherOptions as ChokidarFileWatcherOptions;
    const watcherOptions = {
      ignored: [
        '**/{.git,node_modules}/**',
        new RegExp(this._options.config?.output?.path),
        ...(Array.isArray(ignored) ? ignored : [ignored])
      ],
      ignoreInitial: true,
      ignorePermissionErrors: true
    };

    return watcherOptions;
  }

  normalizeWatchLogger() {
    const outDir = this._options.config.output.path;
    this._logger.info(`Watching for changes`);
    this._logger.info(
      `Ignoring changes in "**/{.git,node_modules}/**" | "${outDir}"`
    );
  }
}
