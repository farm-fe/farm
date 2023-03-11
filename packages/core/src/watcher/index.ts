import chokidar, { FSWatcher } from 'chokidar';
import path from 'path';
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
      serverOrCompiler.hmrEngine?.onUpdate((updateResult) => {
        updateResult.added.forEach((addedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            addedModule
          );
          this._watcher.add(path.join(this._root, resolvedPath));
        });
        updateResult.removed.forEach((removedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            removedModule
          );
          this._watcher.unwatch(path.join(this._root, resolvedPath));
        });
      });
    }

    this._watcher.on('change', (path) => {
      if (serverOrCompiler instanceof DevServer) {
        serverOrCompiler.hmrEngine.hmrUpdate(path);
      } else {
        // TODO update and emit the result
        compiler.updateSync([path]);
        compiler.writeResourcesToDisk();
      }
    });
  }
}
