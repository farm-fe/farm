import EventEmitter from 'node:events';
import { existsSync } from 'node:fs';
import path from 'node:path';
import chokidar from 'chokidar';
import type { FSWatcher, WatchOptions } from 'chokidar';
import glob from 'fast-glob';
import { Compiler } from '../compiler/index.js';
import type { ResolvedUserConfig } from '../config/index.js';
import { createInlineCompiler } from '../index.js';
// import { Server } from '../server/index.js';
import { Server } from '../server/index.js';
import type { JsUpdateResult } from '../types/binding.js';
import { createDebugger } from '../utils/debug.js';
import {
  Logger,
  arraify,
  compilerHandler,
  getCacheDir
} from '../utils/index.js';

interface ImplFileWatcher {
  // watch(): Promise<void>;
}

export const debugWatcher = createDebugger('farm:watcher');

// TODO remove FileWatcher
export default class Watcher implements ImplFileWatcher {
  private watchedFiles = new Set<string>();
  resolvedWatchOptions: WatchOptions;
  watcher: FSWatcher;
  extraWatchedFiles: string[];

  constructor(
    public config: ResolvedUserConfig,
    private logger: Logger = new Logger()
  ) {
    this.resolveChokidarOptions();
  }

  getInternalWatcher() {
    return this.watcher;
  }

  filterWatchFile(file: string, root: string): boolean {
    const separator = process.platform === 'win32' ? '\\' : '/';
    return (
      !file.startsWith(`${root}${separator}`) &&
      !file.includes('\0') &&
      existsSync(file)
    );
  }

  getExtraWatchedFiles(compiler?: Compiler | null) {
    this.extraWatchedFiles = [
      ...compiler.resolvedModulePaths(this.config.root),
      ...compiler.resolvedWatchPaths()
    ].filter((file) => this.filterWatchFile(file, this.config.root));
    return this.extraWatchedFiles;
  }

  watchExtraFiles() {
    this.extraWatchedFiles.forEach((file) => {
      if (!this.watchedFiles.has(file)) {
        this.watcher.add(file);
        this.watchedFiles.add(file);
      }
    });
  }

  // async watch() {
  //   const compiler = this.getCompiler();

  //   const handlePathChange = async (path: string) => {
  //     if (this.close) return;

  //     try {
  //       if (this.compiler instanceof NewServer && this.compiler.getCompiler()) {
  //         await this.compiler.hmrEngine.hmrUpdate(path);
  //       }

  //       if (
  //         this.compiler instanceof Compiler &&
  //         this.compiler.hasModule(path)
  //       ) {
  //         await compilerHandler(
  //           async () => {
  //             const result = await compiler.update([path], true);
  //             this.handleUpdateFinish(result, compiler);
  //             compiler.writeResourcesToDisk();
  //           },
  //           this.config,
  //           this.logger,
  //           { clear: true }
  //         );
  //       }
  //     } catch (error) {
  //       this.logger.error(error);
  //     }
  //   };

  //   const filesToWatch = [this.config.root, ...this.getExtraWatchedFiles()];
  //   this.watchedFiles = new Set(filesToWatch);
  //   this.watcher ??= createWatcher(this.config, filesToWatch);

  //   this.watcher.on('change', (path) => {
  //     if (this.close) return;
  //     handlePathChange(path);
  //   });

  //   if (this.compiler instanceof NewServer) {
  //     this.compiler.hmrEngine?.onUpdateFinish((result) =>
  //       this.handleUpdateFinish(result, compiler)
  //     );
  //   }
  // }

  // async watchConfigs(callback: (files: string[]) => void) {
  //   const filesToWatch = Array.from([
  //     ...(this.config.envFiles ?? []),
  //     ...(this.config.configFileDependencies ?? []),
  //     ...(this.config.configFilePath ? [this.config.configFilePath] : [])
  //   ]).filter((file) => file && existsSync(file));
  //   const chokidarOptions = {
  //     awaitWriteFinish:
  //       process.platform === 'linux'
  //         ? undefined
  //         : {
  //             stabilityThreshold: 10,
  //             pollInterval: 80
  //           }
  //   };
  //   this.watcher ??= createWatcher(this.config, filesToWatch, chokidarOptions);

  //   this.watcher.on('change', (path) => {
  //     if (this.close) return;
  //     if (filesToWatch.includes(path)) {
  //       callback([path]);
  //     }
  //   });
  //   return this;
  // }

  // private handleUpdateFinish(updateResult: JsUpdateResult, compiler: Compiler) {
  //   const addedFiles = [
  //     ...updateResult.added,
  //     ...updateResult.extraWatchResult.add
  //   ].map((addedModule) =>
  //     compiler.transformModulePath(this.config.root, addedModule)
  //   );

  //   const filteredAdded = addedFiles.filter((file) =>
  //     this.filterWatchFile(file, this.config.root)
  //   );

  //   if (filteredAdded.length > 0) {
  //     this.watcher.add(filteredAdded);
  //   }
  // }

  async createWatcher() {
    const compiler = await createInlineCompiler(this.config, {
      progress: false
    });
    // TODO type error here
    // @ts-ignore
    const enabledWatcher = this.config.watch !== null;
    const files = [this.config.root, ...this.getExtraWatchedFiles(compiler)];

    this.watcher = enabledWatcher
      ? (chokidar.watch(files, this.resolvedWatchOptions) as FSWatcher)
      : new NoopWatcher(this.resolvedWatchOptions);
  }

  resolveChokidarOptions() {
    // TODO use config.cacheDir
    const cacheDir = getCacheDir(
      this.config.root,
      this.config.compilation.persistentCache
    );

    // TODO type error here
    // @ts-ignore
    const userWatchOptions = this.config.server.watch;
    const { ignored: ignoredList, ...otherOptions } = userWatchOptions ?? {};
    const ignored: WatchOptions['ignored'] = [
      '**/.git/**',
      // TODO node_modules 这块的处理逻辑 以及是否会影响性能
      '**/node_modules/**',
      '**/test-results/**', // Playwright
      glob.escapePath(
        path.resolve(this.config.root, this.config.compilation.output.path)
      ) + '/**',
      glob.escapePath(cacheDir) + '/**',
      ...arraify(ignoredList || [])
    ];

    this.resolvedWatchOptions = {
      ignored,
      ignoreInitial: true,
      ignorePermissionErrors: true,
      awaitWriteFinish:
        process.platform === 'linux'
          ? undefined
          : {
              stabilityThreshold: 10,
              pollInterval: 10
            },
      ...otherOptions
    };
  }

  async close() {
    if (this.watcher) {
      debugWatcher?.('close watcher');
      await this.watcher.close();
      this.watcher = null;
    }
  }
}

class NoopWatcher extends EventEmitter implements FSWatcher {
  constructor(public options: WatchOptions) {
    super();
  }

  add() {
    return this;
  }

  unwatch() {
    return this;
  }

  getWatched() {
    return {};
  }

  ref() {
    return this;
  }

  unref() {
    return this;
  }

  async close() {
    // noop
  }
}
