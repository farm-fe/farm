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

  watch(callback: (file: string) => void) {
    async function handle(file: string) {
      callback(file);
    }

    this.watcher = new JsFileWatcher((paths: string[]) => {
      if (this._close) return;
      paths.forEach(handle);
    });

    this.watcher.watch([
      ...(this.options.config.config.envFiles ?? []),
      ...(this.options.userConfig.configFileDependencies ?? []),
      this.options.userConfig.resolveConfigPath
    ]);
    return this;
  }

  close() {
    this._close = true;
    this.watcher = null;
  }
}
