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
    console.log(serverOrCompiler, '编译器');
    console.log(this._root);
    console.log(compiler);

    console.log(compiler.resolvedModulePaths(this._root));

    this._watcher = chokidar.watch(compiler.resolvedModulePaths(this._root), {
      ignored: this._options.ignores,
    });
    console.log('开始监听了啊');

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

    this._watcher.on('change', async (path) => {
      console.log(66666);

      if (serverOrCompiler instanceof DevServer) {
        serverOrCompiler.hmrEngine.hmrUpdate(path);
      } else {
        // TODO update and emit the result
        console.log('编译', path);
        await compiler.update([path]);
        compiler.writeResourcesToDisk();
      }
    });
  }
}
