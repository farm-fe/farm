import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { Config, JsFileWatcher } from '../../binding/index.js';
import { compilerHandler, DefaultLogger } from '../utils/index.js';
import debounce from 'lodash.debounce';
import { DEFAULT_HMR_OPTIONS } from '../index.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: JsFileWatcher;
  private _logger: DefaultLogger;
  private _awaitWriteFinish: number;

  constructor(
    public serverOrCompiler: DevServer | Compiler,
    public options?: Config
  ) {
    this._root = options.config.root;
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
        this._logger.error(error);
      }
    };

    if (process.platform === 'win32') {
      handlePathChange = debounce(handlePathChange, 3);
    }

    this._watcher = new JsFileWatcher((paths: string[]) => {
      paths.forEach(handlePathChange);
    });

    this._watcher.watch([
      ...compiler.resolvedModulePaths(this._root),
      ...compiler.resolveWatchPaths()
    ]);

    if (this.serverOrCompiler instanceof DevServer) {
      // chokidar.watch(
      //   compiler.resolvedModulePaths(this._root),
      //   this.serverOrCompiler.config.hmr.watchOptions
      // );
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

        this._watcher.watch(added);

        // const removed = updateResult.removed.map((removedModule) => {
        //   const resolvedPath = compiler.transformModulePath(
        //     this._root,
        //     removedModule
        //   );
        //   return resolvedPath;
        // });

        // this._watcher.unwatch(removed);
      });
    }

    if (this.serverOrCompiler instanceof Compiler) {
      // const watcherOptions = this.resolvedWatcherOptions();
      // this._watcher = chokidar.watch(
      //   compiler.resolvedModulePaths(this._root),
      //   watcherOptions as ChokidarFileWatcherOptions
      // );
    }

    // this._watcher.on('change', );
  }

  private getCompilerFromServerOrCompiler(
    serverOrCompiler: DevServer | Compiler
  ): Compiler {
    return serverOrCompiler instanceof DevServer
      ? serverOrCompiler.getCompiler()
      : serverOrCompiler;
  }

  // private resolvedWatcherOptions() {
  //   const { watch: watcherOptions, output } = this.options.config;
  //   const userWatcherOptions = isObject(watcherOptions) ? watcherOptions : {};
  //   const { ignored = [] } = userWatcherOptions as ChokidarFileWatcherOptions;
  //   const resolveWatcherOptions = {
  //     ignoreInitial: true,
  //     ignorePermissionErrors: true,
  //     ...watcherOptions,
  //     ignored: [
  //       '**/{.git,node_modules}/**',
  //       output?.path,
  //       ...(Array.isArray(ignored) ? ignored : [ignored])
  //     ]
  //   };
  //   // TODO other logger info
  //   this._logger.info(`Watching for changes`);
  //   this._logger.info(
  //     `Ignoring changes in ${resolveWatcherOptions.ignored
  //       .map((v: string | RegExp) => '"' + v + '"')
  //       .join(' | ')}`
  //   );
  //   return resolveWatcherOptions;
  // }
}

export async function restartServer(server: DevServer) {
  await server.close();
}
