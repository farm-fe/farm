import { Config, JsFileWatcher } from '../../binding/index.js';
import { ResolvedUserConfig } from '../config/index.js';

interface WatcherOptions {
  config: Config;
  userConfig: ResolvedUserConfig;
}

export class ConfigWatcher {
  private watcher: JsFileWatcher;
  private _close = false;

  constructor(private options: WatcherOptions) {}

  watch(callback: (file: string[]) => void) {
    async function handle(file: string[]) {
      callback(file);
    }

    const watchedFiles = [
      ...(this.options.config.config.envFiles ?? []),
      ...(this.options.userConfig.configFileDependencies ?? []),
      this.options.userConfig.resolveConfigPath
    ];

    this.watcher = new JsFileWatcher((paths: string[]) => {
      if (this._close) return;
      const filteredPaths = paths.filter((path) => watchedFiles.includes(path));

      if (!filteredPaths.length) return;
      handle(filteredPaths);
    });

    this.watcher.watch(watchedFiles);
    return this;
  }

  close() {
    this._close = true;
    this.watcher = null;
  }
}
