import chokidar, { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';

import { DevServer } from '../server/index.js';

export class FileWatcher {
  private _root: string;
  private _watcher: FSWatcher;

  constructor(root: string, config?: { ignores?: string[] }) {
    this._root = root;

    this._watcher = chokidar.watch(this._root, {
      ignored: config?.ignores ?? [],
    });
  }

  watch(serverOrCompiler: DevServer | Compiler) {
    this._watcher.on('change', (path) => {
      console.log(path);

      if (serverOrCompiler instanceof DevServer) {
        serverOrCompiler.hmrEngine.hmrUpdate(path);
      } else {
        // TODO update and emit the result
        serverOrCompiler.updateSync([path]);
      }
    });
  }
}
