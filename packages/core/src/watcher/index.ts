import chalk from 'chalk';
import chokidar, { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';

import { DevServer } from '../server/index.js';

export interface FileWatcherOptions {
  ignores?: string[] | any;
}

export class FileWatcher {
  private _root: string;
  private _watcher: FSWatcher;
  private _options: FileWatcherOptions;

  constructor(root: string, config?: FileWatcherOptions) {
    this._root = root;
    this._options = config ?? {};
  }

  async watch(serverOrCompiler: DevServer | Compiler) {
    const compiler =
      serverOrCompiler instanceof DevServer
        ? serverOrCompiler.getCompiler()
        : serverOrCompiler;

    this._watcher = chokidar.watch(
      serverOrCompiler instanceof DevServer
        ? compiler.resolvedModulePaths(this._root)
        : [this._root],
      {
        ignored: this._options.ignores
      }
    );
    if (serverOrCompiler instanceof DevServer) {
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
      await compiler.compile();
      compiler.writeResourcesToDisk();
    }

    this._watcher.on('change', async (path) => {
      if (serverOrCompiler instanceof DevServer) {
        serverOrCompiler.hmrEngine.hmrUpdate(path);
      } else {
        // TODO update and emit the result
        const start = Date.now();
        await compiler.update([path]);
        compiler.writeResourcesToDisk();
        console.warn(
          `Build completed in ${chalk.green(
            `${Date.now() - start}ms`
          )}! Resources emitted to ${chalk
            .green
            // normalizedConfig.config.output.path
            ()}.`
        );
      }
    });
  }
}
