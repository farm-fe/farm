import { JsFileWatcher } from '../../binding/index.js';
import { ResolvedUserConfig } from '../config/index.js';

export class ConfigWatcher {
  private watcher: JsFileWatcher;
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

    const watchedFiles = Array.from(watchedFilesSet).filter(Boolean);

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
