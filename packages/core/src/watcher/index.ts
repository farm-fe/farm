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
import {
  arraify,
  bold,
  colors,
  getShortName,
  green,
  normalizePath
} from '../utils/index.js';

import { __FARM_GLOBAL__ } from '../config/_global.js';
import type { ResolvedUserConfig } from '../config/index.js';
import type {
  JsUpdateResult,
  PersistentCacheConfig
} from '../types/binding.js';

export const debugWatcher = createDebugger('farm:watcher');

interface IWatcher {
  resolvedWatchOptions: WatchOptions;
  extraWatchedFiles: string[];
  getInternalWatcher(): FSWatcher;
  filterWatchFile(file: string, root: string): boolean;
  getExtraWatchedFiles(compiler?: Compiler | null): string[];
  watchExtraFiles(): void;
  createWatcher(): Promise<void>;
  resolveChokidarOptions(): void;
  close(): Promise<void>;
}

export default class Watcher extends EventEmitter implements IWatcher {
  private watchedFiles = new Set<string>();
  public resolvedWatchOptions: WatchOptions;
  public extraWatchedFiles: string[] = [];
  private _watcher: FSWatcher;

  constructor(public config: ResolvedUserConfig) {
    super();
    this.resolveChokidarOptions();
  }

  on(
    event: 'add' | 'addDir' | 'change' | 'unlink' | 'unlinkDir',
    listener: (path: string) => void
  ): this {
    this._watcher.on(event, listener);
    return this;
  }

  add(paths: string | ReadonlyArray<string>): this {
    this._watcher.add(paths);
    return this;
  }

  unwatch(paths: string | ReadonlyArray<string>): this {
    this._watcher.unwatch(paths);
    return this;
  }

  getWatched(): { [directory: string]: string[] } {
    return this._watcher.getWatched();
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
        this._watcher.add(file);
        this.watchedFiles.add(file);
      }
    });
  }

  async createWatcher() {
    const compiler = await createInlineCompiler(this.config, {
      progress: false
    });
    const enabledWatcher = this.config.watch !== null;
    const files = [this.config.root, ...this.getExtraWatchedFiles(compiler)];

    this._watcher = enabledWatcher
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
    if (this._watcher) {
      debugWatcher?.('close watcher');
      await this._watcher.close();
      this._watcher = null;
    }
  }

  isConfigFilesChanged(file: string): boolean {
    file = normalizePath(file);
    const shortFile = getShortName(file, this.config.root);
    const isConfigFile = this.config.configFilePath === file;
    const isConfigDependencyFile = this.config.configFileDependencies.some(
      (name) => file === name
    );
    const isEnvFile = this.config.envFiles.some((name) => file === name);

    if (isConfigFile || isConfigDependencyFile || isEnvFile) {
      __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
      this.config.logger.info(
        `${bold(green(shortFile))} changed, Server is being restarted`,
        true
      );
      this.config.logger.info(`[config change] ${colors.dim(file)}`);
      return true;
    }

    return false;
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

export async function watchFileChangeAndRebuild(
  resolvedUserConfig: ResolvedUserConfig,
  compiler: Compiler,
  onRebuild: () => Promise<void>
) {
  const logger = resolvedUserConfig.logger;
  const watcher = new Watcher(resolvedUserConfig);
  await watcher.createWatcher();

  watcher.on('add', async (_file) => {
    // TODO
  });

  watcher.on('unlink', async (_file) => {
    // TODO
  });

  watcher.on('change', async (file: string | string[] | any) => {
    if (!compiler.hasModule(file)) {
      return;
    }

    file = normalizePath(file);

    if (watcher.isConfigFilesChanged(file)) {
      // rebuild the project
      return onRebuild();
    }

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
        watcher.add(filteredAdded);
      }
    };

    try {
      const start = performance.now();
      const result = await compiler.update([file], true, false, false);
      const elapsedTime = Math.floor(performance.now() - start);

      logger.info(
        `update completed in ${bold(
          green(`${logger.formatTime(elapsedTime)}`)
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
