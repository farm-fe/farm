import chokidar, { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';

import { DevServer } from '../server/index.js';

export interface FileWatcherOptions {
  ignores?: string[];
}

export class FileWatcher {
  private _root: string;
  private _watcher: FSWatcher;
  private _options: FileWatcherOptions;

  constructor(root: string, config?: FileWatcherOptions) {
    this._root = root;
    this._options = config ?? {};
  }

  watch(serverOrCompiler: DevServer | Compiler) {
    const compiler =
      serverOrCompiler instanceof DevServer
        ? serverOrCompiler.getCompiler()
        : serverOrCompiler;

    this._watcher = chokidar.watch(compiler.resolvedModulePaths(this._root), {
      ignored: this._options.ignores,
    });

    if (serverOrCompiler instanceof DevServer) {
      serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
        updateResult.added.forEach((addedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            addedModule
          );
          this._watcher.add(resolvedPath);
        });
        updateResult.removed.forEach((removedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            removedModule
          );
          this._watcher.unwatch(resolvedPath);
        });
      });
    }

    this._watcher.on('change', async (path) => {
      if (serverOrCompiler instanceof DevServer) {
        serverOrCompiler.hmrEngine.hmrUpdate(path);
      } else {
        // TODO update and emit the result
        await compiler.update([path]);
        compiler.writeResourcesToDisk();
      }
    });
  }
}
