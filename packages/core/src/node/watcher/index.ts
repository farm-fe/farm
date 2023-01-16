import chokidar, { FSWatcher } from 'chokidar';
import merge from 'lodash.merge';
import { Compiler } from '../compiler/index.js';

import { UserHmrConfig } from '../config/index.js';
import { DevServer } from '../server/index.js';

const DEFAULT_WATCHER_CONFIG: Required<UserHmrConfig> = {
  ignores: [],
};

export class FileWatcher {
  private _config: UserHmrConfig;
  private _root: string;
  private _watcher: FSWatcher;

  constructor(root: string, config: UserHmrConfig = {}) {
    this._config = merge(DEFAULT_WATCHER_CONFIG, config);
    this._root = root;

    this._watcher = chokidar.watch(this._root, {
      ignored: this._config.ignores,
    });
  }

  watch(_compiler: Compiler, _devServer?: DevServer) {
    this._watcher.on('change', (path) => {
      console.log(path);
    });
  }
}
