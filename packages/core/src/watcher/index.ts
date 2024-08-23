import { createRequire } from 'node:module';

import { FSWatcher } from 'chokidar';

import { Compiler } from '../compiler/index.js';
import { Server } from '../server/index.js';
import { Logger, compilerHandler } from '../utils/index.js';

import { existsSync } from 'node:fs';
import type { ResolvedUserConfig } from '../config/index.js';
import type { JsUpdateResult } from '../types/binding.js';
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
    public serverOrCompiler: Server | Compiler,
    public options: ResolvedUserConfig,
    private _logger: Logger
  ) {
    this._root = options.root;
  }

  getInternalWatcher() {
    return this._watcher;
  }

  filterWatchFile(file: string, root: string): boolean {
    const suffix = process.platform === 'win32' ? '\\' : '/';

    return (
      !file.startsWith(`${root}${suffix}`) &&
      !file.includes('\0') &&
      existsSync(file)
    );
  }

  getExtraWatchedFiles() {
    const compiler = this.getCompilerFromServerOrCompiler(
      this.serverOrCompiler
    );

    return [
      ...compiler.resolvedModulePaths(this._root),
      ...compiler.resolvedWatchPaths()
    ].filter((file) => this.filterWatchFile(file, this._root));
  }

  watchExtraFiles() {
    const files = this.getExtraWatchedFiles();

    for (const file of files) {
      if (!this._watchedFiles.has(file)) {
        this._watcher.add(file);
        this._watchedFiles.add(file);
      }
    }
  }

  async watch() {
    // Determine how to compile the project
    const compiler = this.getCompilerFromServerOrCompiler(
      this.serverOrCompiler
    );

    const handlePathChange = async (path: string): Promise<void> => {
      if (this._close) {
        return;
      }

      try {
        // if (this.serverOrCompiler instanceof Server) {
        // @ts-ignore
        if (this.serverOrCompiler.compiler) {
          // @ts-ignore
          await this.serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (
          this.serverOrCompiler instanceof Compiler &&
          this.serverOrCompiler.hasModule(path)
        ) {
          compilerHandler(
            async () => {
              const result = await compiler.update([path], true);
              handleUpdateFinish(result);
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

    const watchedFiles = this.getExtraWatchedFiles();

    const files = [this.options.root, ...watchedFiles];
    this._watchedFiles = new Set(files);
    this._watcher = createWatcher(this.options, files);

    this._watcher.on('change', (path) => {
      if (this._close) return;
      handlePathChange(path);
    });

    const handleUpdateFinish = (updateResult: JsUpdateResult) => {
      const added = [
        ...updateResult.added,
        ...updateResult.extraWatchResult.add
      ].map((addedModule) => {
        const resolvedPath = compiler.transformModulePath(
          this._root,
          addedModule
        );
        return resolvedPath;
      });
      const filteredAdded = added.filter((file) =>
        this.filterWatchFile(file, this._root)
      );

      if (filteredAdded.length > 0) {
        this._watcher.add(filteredAdded);
      }
    };

    if (this.serverOrCompiler instanceof Server) {
      this.serverOrCompiler.hmrEngine?.onUpdateFinish(handleUpdateFinish);
    }
  }

  private getCompilerFromServerOrCompiler(
    serverOrCompiler: Server | Compiler
  ): Compiler {
    // @ts-ignore
    return serverOrCompiler.getCompiler
      ? // @ts-ignore
        serverOrCompiler.getCompiler()
      : serverOrCompiler;
  }

  close() {
    this._close = true;
    this._watcher = null;
    this.serverOrCompiler = null;
  }
}

export function clearModuleCache(modulePath: string) {
  const _require = createRequire(import.meta.url);
  delete _require.cache[_require.resolve(modulePath)];
}
