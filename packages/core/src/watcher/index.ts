import { createRequire } from 'node:module';

import { FSWatcher } from 'chokidar';

import { Compiler } from '../compiler/index.js';
import { Server } from '../server/index.js';
import { compilerHandler, Logger } from '../utils/index.js';

import type { ResolvedUserConfig } from '../config/index.js';
import { createWatcher } from './create-watcher.js';
// import { existsSync } from 'node:fs';
import { JsUpdateResult } from '../../binding/binding.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: FSWatcher;
  private _logger: Logger;
  private _close = false;

  constructor(
    public serverOrCompiler: Server | Compiler,
    public options: ResolvedUserConfig
  ) {
    this._root = options.root;
    this._logger = new Logger();
  }

  getInternalWatcher() {
    return this._watcher;
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
        if (this.serverOrCompiler instanceof Server) {
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
            { clear: true }
          );
        }
      } catch (error) {
        this._logger.error(error);
      }
    };

    const watchedFiles = [
      ...compiler.resolvedModulePaths(this._root),
      ...compiler.resolvedWatchPaths()
    ].filter(
      (file) =>
        !file.startsWith(this.options.root) && !file.includes('node_modules/')
    );

    const files = [this.options.root, ...watchedFiles];
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
      const filteredAdded = added.filter(
        (file) => !file.startsWith(this.options.root)
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
    return serverOrCompiler instanceof Server
      ? serverOrCompiler.getCompiler()
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
