import chokidar, { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { DefaultLogger } from '../utils/logger.js';

import { Config } from '../../binding/index.js';
import { isObject } from '../utils/common.js';

import type { WatchOptions as ChokidarFileWatcherOptions } from 'chokidar';
import { compilerHandler } from '../utils/build.js';

interface ImplFileWatcher {
  watch(serverOrCompiler: DevServer | Compiler, config: Config): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
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

    if (serverOrCompiler instanceof DevServer) {
      this._watcher = chokidar.watch(compiler.resolvedModulePaths(this._root));
      serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
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

    if (serverOrCompiler instanceof Compiler) {
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
    const { watch, output } = this._options.config;
    const watchOptionsType = isObject(watch);
    const userWatcherOptions = watchOptionsType ? watch : {};
    const { ignored = [] } = userWatcherOptions as ChokidarFileWatcherOptions;
    const watcherOptions = {
      ignored: [
        '**/{.git,node_modules}/**',
        new RegExp(output?.path),
        ...(Array.isArray(ignored) ? ignored : [ignored])
      ],
      ignoreInitial: true,
      ignorePermissionErrors: true
    };
    this._logger.info(`Watching for changes`);
    this._logger.info(
      `Ignoring changes in ${watcherOptions.ignored
        .map((v) => '"' + v + '"')
        .join(' | ')}`
    );
    return watcherOptions;
  }
}
