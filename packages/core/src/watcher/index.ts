import chokidar, { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { DefaultLogger } from '../utils/logger.js';

import { Config } from '../../binding/index.js';
import { isObject } from '../utils/index.js';

import type { WatchOptions as ChokidarFileWatcherOptions } from 'chokidar';
import { compilerHandler } from '../utils/build.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: FSWatcher;
  private _logger: DefaultLogger;

  constructor(
    public serverOrCompiler: DevServer | Compiler,
    public options?: Config
  ) {
    this._root = options.config.root;
    this._logger = new DefaultLogger();
  }

  async watch() {
    // Determine how to compile the project
    const compiler = this.getCompilerFromServerOrCompiler(
      this.serverOrCompiler
    );

    if (this.serverOrCompiler instanceof DevServer) {
      this._watcher = chokidar.watch(compiler.resolvedModulePaths(this._root));
      this.serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
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

    if (this.serverOrCompiler instanceof Compiler) {
      const watcherOptions = this.resolvedWatcherOptions();
      this._watcher = chokidar.watch(
        compiler.resolvedModulePaths(this._root),
        watcherOptions as ChokidarFileWatcherOptions
      );
    }

    this._watcher.on('change', async (path: string) => {
      try {
        if (this.serverOrCompiler instanceof DevServer) {
          await this.serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (this.serverOrCompiler instanceof Compiler) {
          compilerHandler(async () => {
            await compiler.update([path], true);
            compiler.writeResourcesToDisk();
          }, this.options);
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
    const { watch: watcherOptions, output } = this.options.config;
    const userWatcherOptions = isObject(watcherOptions) ? watcherOptions : {};
    const { ignored = [] } = userWatcherOptions as ChokidarFileWatcherOptions;
    const resolveWatcherOptions = {
      ignoreInitial: true,
      ignorePermissionErrors: true,
      ...watcherOptions,
      ignored: [
        '**/{.git,node_modules}/**',
        output?.path,
        ...(Array.isArray(ignored) ? ignored : [ignored])
      ]
    };
    // TODO other logger info
    this._logger.info(`Watching for changes`);
    this._logger.info(
      `Ignoring changes in ${resolveWatcherOptions.ignored
        .map((v: string | RegExp) => '"' + v + '"')
        .join(' | ')}`
    );
    return resolveWatcherOptions;
  }
}

export async function restartServer(server: DevServer) {
  await server.close();
}
