import EventEmitter from 'node:events';
import { existsSync } from 'node:fs';
import path from 'node:path';

import chokidar from 'chokidar';
import type { FSWatcher, WatchOptions } from 'chokidar';
import glob from 'fast-glob';

import { Compiler } from '../compiler/index.js';
import { createInlineCompiler } from '../compiler/index.js';
import { createDebugger } from '../utils/debug.js';
import { convertErrorMessage } from '../utils/error.js';
import { arraify, bold, green, normalizePath } from '../utils/index.js';

import type { ResolvedUserConfig } from '../config/index.js';
import type {
  JsUpdateResult,
  PersistentCacheConfig
} from '../types/binding.js';

export const debugWatcher = createDebugger('farm:watcher');

export default class Watcher {
  private watchedFiles = new Set<string>();
  resolvedWatchOptions: WatchOptions;
  watcher: FSWatcher;
  extraWatchedFiles: string[];

  constructor(public config: ResolvedUserConfig) {
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

  async watch() {}

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
    const userWatchOptions = this.config.watch;
    const { ignored: ignoredList, ...otherOptions } =
      typeof userWatchOptions === 'object' ? userWatchOptions : {};
    const cacheDir = (
      this.config.compilation.persistentCache as PersistentCacheConfig
    ).cacheDir;
    const ignored: WatchOptions['ignored'] = [
      '**/.git/**',
      '**/node_modules/**',
      '**/test-results/**', // Playwright
      glob.escapePath(
        path.resolve(this.config.root, this.config.compilation.output.path)
      ) + '/**',
      cacheDir ? glob.escapePath(cacheDir) + '/**' : undefined,
      ...arraify(ignoredList || [])
    ].filter(Boolean);

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

export async function handlerWatcher(
  resolvedUserConfig: ResolvedUserConfig,
  compiler: Compiler
) {
  const logger = resolvedUserConfig.logger;
  const watcher = new Watcher(resolvedUserConfig);
  await watcher.createWatcher();
  watcher.watcher.on('change', async (file: string | string[] | any) => {
    file = normalizePath(file);
    // TODO restart with node side v2.0 we may be think about this feature
    // const shortFile = getShortName(file, resolvedUserConfig.root);
    // const isConfigFile = resolvedUserConfig.configFilePath === file;
    // const isConfigDependencyFile =
    //   resolvedUserConfig.configFileDependencies.some((name) => file === name);
    // const isEnvFile = resolvedUserConfig.envFiles.some((name) => file === name);
    // if (isConfigFile || isConfigDependencyFile || isEnvFile) {
    //   __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
    //   resolvedUserConfig.logger.info(
    //     `${bold(green(shortFile))} changed, Bundler Config is being reloaded`,
    //     true
    //   );
    // TODO then rebuild node side
    // }
    const handleUpdateFinish = (updateResult: JsUpdateResult) => {
      const added = [
        ...updateResult.added,
        ...updateResult.extraWatchResult.add
      ].map((addedModule) => {
        const resolvedPath = compiler.transformModulePath(
          resolvedUserConfig.root,
          addedModule
        );
        return resolvedPath;
      });

      const filteredAdded = added.filter((file) =>
        watcher.filterWatchFile(file, resolvedUserConfig.root)
      );

      if (filteredAdded.length > 0) {
        watcher.watcher.add(filteredAdded);
      }
    };

    try {
      const start = performance.now();
      const result = await compiler.update([file], true);
      const elapsedTime = Math.floor(performance.now() - start);
      logger.info(
        `update completed in ${bold(
          green(`${logger.formatExecutionTime(elapsedTime)}ms`)
        )} Resources emitted to ${bold(
          green(resolvedUserConfig.compilation.output.path)
        )}.`
      );
      handleUpdateFinish(result);
      compiler.writeResourcesToDisk();
    } catch (error) {
      resolvedUserConfig.logger.error(
        `Farm Update Error: ${convertErrorMessage(error)}`
      );
    }
  });
}
