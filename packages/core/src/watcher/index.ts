import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';
import type { ResolvedUserConfig } from '../config/index.js';
// import { Server } from '../server/index.js';
import { newServer } from '../newServer/index.js';
import type { JsUpdateResult } from '../types/binding.js';
import { Logger, compilerHandler } from '../utils/index.js';
import { createWatcher } from './create-watcher.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: FSWatcher;
  private _close = false;
  private _watchedFiles = new Set<string>();

  constructor(
    public serverOrCompiler: newServer | Compiler,
    public options: ResolvedUserConfig,
    private _logger: Logger
  ) {
    this._root = options.root;
  }

  getInternalWatcher() {
    return this._watcher;
  }

  filterWatchFile(file: string, root: string): boolean {
    const separator = process.platform === 'win32' ? '\\' : '/';
    return (
      !file.startsWith(`${root}${separator}`) &&
      !file.includes('\0') &&
      existsSync(file)
    );
  }

  getExtraWatchedFiles() {
    const compiler = this.getCompiler();
    return [
      ...compiler.resolvedModulePaths(this._root),
      ...compiler.resolvedWatchPaths()
    ].filter((file) => this.filterWatchFile(file, this._root));
  }

  watchExtraFiles() {
    this.getExtraWatchedFiles().forEach((file) => {
      if (!this._watchedFiles.has(file)) {
        this._watcher.add(file);
        this._watchedFiles.add(file);
      }
    });
  }

  async watch() {
    const compiler = this.getCompiler();

    const handlePathChange = async (path: string) => {
      if (this._close) return;

      try {
        if (
          this.serverOrCompiler instanceof newServer &&
          this.serverOrCompiler.getCompiler()
        ) {
          await this.serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (
          this.serverOrCompiler instanceof Compiler &&
          this.serverOrCompiler.hasModule(path)
        ) {
          await compilerHandler(
            async () => {
              const result = await compiler.update([path], true);
              this.handleUpdateFinish(result, compiler);
              compiler.writeResourcesToDisk();
            },
            this.options,
            this._logger,
            { clear: true }
          );
        }
      } catch (error) {
        this._logger.error(error);
      }
    };

    const filesToWatch = [this.options.root, ...this.getExtraWatchedFiles()];
    this._watchedFiles = new Set(filesToWatch);
    this._watcher ??= createWatcher(this.options, filesToWatch);

    this._watcher.on('change', (path) => {
      if (this._close) return;
      handlePathChange(path);
    });

    if (this.serverOrCompiler instanceof newServer) {
      this.serverOrCompiler.hmrEngine?.onUpdateFinish((result) =>
        this.handleUpdateFinish(result, compiler)
      );
    }
  }

  async watchConfigs(callback: (files: string[]) => void) {
    const filesToWatch = Array.from([
      ...(this.options.envFiles ?? []),
      ...(this.options.configFileDependencies ?? []),
      ...(this.options.configFilePath ? [this.options.configFilePath] : [])
    ]).filter((file) => file && existsSync(file));
    const chokidarOptions = {
      awaitWriteFinish:
        process.platform === 'linux'
          ? undefined
          : {
              stabilityThreshold: 10,
              pollInterval: 80
            }
    };
    this._watcher ??= createWatcher(
      this.options,
      filesToWatch,
      chokidarOptions
    );

    this._watcher.on('change', (path) => {
      if (this._close) return;
      if (filesToWatch.includes(path)) {
        callback([path]);
      }
    });
    return this;
  }

  private handleUpdateFinish(updateResult: JsUpdateResult, compiler: Compiler) {
    const addedFiles = [
      ...updateResult.added,
      ...updateResult.extraWatchResult.add
    ].map((addedModule) =>
      compiler.transformModulePath(this._root, addedModule)
    );

    const filteredAdded = addedFiles.filter((file) =>
      this.filterWatchFile(file, this._root)
    );

    if (filteredAdded.length > 0) {
      this._watcher.add(filteredAdded);
    }
  }

  private getCompiler(): Compiler {
    return this.serverOrCompiler instanceof newServer
      ? this.serverOrCompiler.getCompiler()
      : this.serverOrCompiler;
  }

  async close() {
    if (this._watcher) {
      this._close = true;
      await this._watcher.close();
      this._watcher = null;
    }
    this.serverOrCompiler = null;
  }
}

export function clearModuleCache(modulePath: string) {
  const _require = createRequire(import.meta.url);
  delete _require.cache[_require.resolve(modulePath)];
}
