import { existsSync } from 'fs';
import { FSWatcher } from 'chokidar';
import { ResolvedUserConfig } from '../config/index.js';
import { createWatcher } from './create-watcher.js';

export class ConfigWatcher {
  private watcher: FSWatcher;
  private _close = false;

  constructor(private resolvedUserConfig: ResolvedUserConfig) {
    if (!resolvedUserConfig) {
      throw new Error(
        'Invalid resolvedUserConfig provided to Farm JsConfigWatcher'
      );
    }
  }

  watch(callback: (file: string[]) => void) {
    async function handle(file: string[]) {
      callback(file);
    }

    const watchedFilesSet = new Set<string>([
      ...(this.resolvedUserConfig.envFiles ?? []),
      ...(this.resolvedUserConfig.configFileDependencies ?? []),
      ...(this.resolvedUserConfig.configFilePath
        ? [this.resolvedUserConfig.configFilePath]
        : [])
    ]);

    const watchedFiles = Array.from(watchedFilesSet).filter(
      (file) => file && existsSync(file)
    );
    const chokidarOptions = {
      awaitWriteFinish:
        process.platform === 'linux'
          ? undefined
          : {
              stabilityThreshold: 10,
              pollInterval: 80
            }
    };
    this.watcher = createWatcher(
      this.resolvedUserConfig,
      watchedFiles,
      chokidarOptions
    );

    this.watcher.on('change', (path) => {
      if (this._close) return;
      if (watchedFiles.includes(path)) {
        handle([path]);
      }
    });
    return this;
  }

  close() {
    this._close = true;
    this.watcher = null;
  }
}
