import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { Config, JsFileWatcher } from '../../binding/index.js';
import { compilerHandler, DefaultLogger } from '../utils/index.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: JsFileWatcher;
  private _logger: DefaultLogger;

  constructor(
    public serverOrCompiler: DevServer | Compiler,
    public options?: Config
  ) {
    this._root = options.config.root;
    this._logger = new DefaultLogger();
  }

  async watch() {
    // Determine how to compile the project
    const compiler = this.getCompilerFromServerOrCompiler(
      this.serverOrCompiler
    );

    this._watcher = new JsFileWatcher((paths: string[]) => {
      console.log(arguments);
      const handlePathChange = async (path: string): Promise<void> => {
        try {
          if (this.serverOrCompiler instanceof DevServer) {
            await this.serverOrCompiler.hmrEngine.hmrUpdate(path);
          }

          if (this.serverOrCompiler instanceof Compiler) {
            compilerHandler(async () => {
              await compiler.update([path], true);
              compiler.writeResourcesToDisk();
            }, this.options);
          }
        } catch (error) {
          this._logger.error(error);
        }
      };
      paths.forEach(handlePathChange);
    });

    compiler.resolvedModulePaths(this._root).forEach((path) => {
      try {
        this._watcher.watch(path);
      } catch (error) {
        // ignore error
      }
    });

    if (this.serverOrCompiler instanceof DevServer) {
      // chokidar.watch(
      //   compiler.resolvedModulePaths(this._root),
      //   this.serverOrCompiler.config.hmr.watchOptions
      // );
      this.serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
        const added = updateResult.added.map((addedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            addedModule
          );
          return resolvedPath;
        });
        added.forEach((path) => {
          try {
            this._watcher.watch(path);
          } catch (error) {
            // ignore error
          }
        });
        // this._watcher.add(added);

        const removed = updateResult.removed.map((removedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            removedModule
          );
          return resolvedPath;
        });

        // this._watcher.unwatch(removed);
        removed.forEach((path) => {
          try {
            this._watcher.unwatch(path);
          } catch (error) {
            // ignore error
          }
        });
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
