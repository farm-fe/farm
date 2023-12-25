import { createRequire } from 'node:module';

import debounce from 'lodash.debounce';

import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { JsFileWatcher } from '../../binding/index.js';
import { compilerHandler, DefaultLogger } from '../utils/index.js';
import { DEFAULT_HMR_OPTIONS } from '../index.js';

import type { ResolvedUserConfig } from '../config/index.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: JsFileWatcher;
  private _logger: DefaultLogger;
  private _awaitWriteFinish: number;
  private _close = false;

  constructor(
    public serverOrCompiler: DevServer | Compiler,
    public options: ResolvedUserConfig
  ) {
    this._root = options.root;
    this._awaitWriteFinish = DEFAULT_HMR_OPTIONS.watchOptions.awaitWriteFinish;

    if (serverOrCompiler instanceof DevServer) {
      this._awaitWriteFinish =
        serverOrCompiler.config.hmr.watchOptions.awaitWriteFinish ??
        this._awaitWriteFinish;
    } else if (serverOrCompiler instanceof Compiler) {
      this._awaitWriteFinish =
        serverOrCompiler.config.config?.watch?.watchOptions?.awaitWriteFinish ??
        this._awaitWriteFinish;
    }

    this._logger = new DefaultLogger();
  }

  async watch() {
    // Determine how to compile the project
    const compiler = this.getCompilerFromServerOrCompiler(
      this.serverOrCompiler
    );

    let handlePathChange = async (path: string): Promise<void> => {
      if (this._close) {
        return;
      }

      try {
        if (this.serverOrCompiler instanceof DevServer) {
          await this.serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (
          this.serverOrCompiler instanceof Compiler &&
          this.serverOrCompiler.hasModule(path)
        ) {
          compilerHandler(async () => {
            await compiler.update([path], true);
            compiler.writeResourcesToDisk();
          }, this.options);
        }
      } catch (error) {
        console.log('我这是不是走了两遍 error 啊', error);
        this._logger.error(error);
      }
    };

    if (process.platform === 'win32') {
      handlePathChange = debounce(handlePathChange, this._awaitWriteFinish);
    }

    this._watcher = new JsFileWatcher((paths: string[]) => {
      paths.forEach(handlePathChange);
    });

    this._watcher.watch([
      ...compiler.resolvedModulePaths(this._root),
      ...compiler.resolvedWatchPaths()
    ]);

    if (this.serverOrCompiler instanceof DevServer) {
      this.serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
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
        this._watcher.watch([...new Set(added)]);
      });
    }
  }

  private getCompilerFromServerOrCompiler(
    serverOrCompiler: DevServer | Compiler
  ): Compiler {
    return serverOrCompiler instanceof DevServer
      ? serverOrCompiler.getCompiler()
      : serverOrCompiler;
  }

  close() {
    this._close = false;
    this._watcher = null;
    this.serverOrCompiler = null;
  }
}

export function clearModuleCache(modulePath: string) {
  const _require = createRequire(import.meta.url);
  delete _require.cache[_require.resolve(modulePath)];
}
