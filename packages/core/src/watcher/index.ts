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

  async watch(serverOrCompiler: DevServer | Compiler, config: any) {
    const compiler =
      serverOrCompiler instanceof DevServer
        ? serverOrCompiler.getCompiler()
        : serverOrCompiler;

    this._watcher = chokidar.watch(
      serverOrCompiler instanceof DevServer
        ? compiler.resolvedModulePaths(this._root)
        : [this._root],
      {
        ignored: this._options.ignores,
        awaitWriteFinish: {
          stabilityThreshold: 300, // 稳定性阈值为 1000ms
          pollInterval: 100 // 轮询间隔为 100ms
        }
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
        const start = Date.now();
        await compiler.update([path], true);
        console.warn(
          `Build completed in ${chalk.green(
            `${Date.now() - start}ms`
          )}! Resources emitted to ${chalk.green(config.config.output.path)}.`
        );
        compiler.writeResourcesToDisk();
      }
    });
  }
}
