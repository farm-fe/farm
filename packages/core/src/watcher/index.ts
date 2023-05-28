import { performance } from 'node:perf_hooks';
import chalk from 'chalk';
import chokidar, { FSWatcher } from 'chokidar';
import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { DefaultLogger } from '../utils/logger.js';

import { Config } from '../../binding/index.js';

export interface FileWatcherOptions {
  ignores?: (string | RegExp)[];
}

export class FileWatcher {
  private _root: string;
  private _watcher: FSWatcher;
  private _options: FileWatcherOptions;
  private _logger: DefaultLogger;

  constructor(root: string, options?: FileWatcherOptions) {
    this._root = root;
    this._logger = new DefaultLogger();
    this._options = options ?? {};
  }

  async watch(serverOrCompiler: DevServer | Compiler, config: Config) {
    const compiler =
      serverOrCompiler instanceof DevServer
        ? serverOrCompiler.getCompiler()
        : serverOrCompiler;

    // this._watcher = chokidar.watch(
    //   serverOrCompiler instanceof DevServer
    //     ? compiler.resolvedModulePaths(this._root)
    //     : [this._root],
    //   {
    //     ignored: this._options.ignores
    //   }
    // );
    this._watcher = chokidar.watch([this._root], {
      ignored: this._options.ignores
    });
    if (serverOrCompiler instanceof DevServer) {
      serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
        const added = updateResult.added.map((addedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            addedModule
          );
          return resolvedPath;
        });
        this._watcher.add(added);

        const removed = updateResult.removed.map((removedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            removedModule
          );
          return resolvedPath;
        });

        this._watcher.unwatch(removed);
      });
    }

    if (serverOrCompiler instanceof Compiler) {
      await compilerHandler(async () => {
        await compiler.compile();
        compiler.writeResourcesToDisk();
      }, config);
      normalizeWatchLogger(this._logger, config);
    }

    this._watcher.on('change', async (path) => {
      try {
        if (serverOrCompiler instanceof DevServer) {
          await serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (serverOrCompiler instanceof Compiler) {
          await compilerHandler(async () => {
            await compiler.update([path], true);
            compiler.writeResourcesToDisk();
          }, config);
        }
      } catch (error) {
        this._logger.error(error);
      }
    });
  }
}

export function normalizeWatchLogger(logger: DefaultLogger, config?: Config) {
  const outDir = config.config.output.path;
  logger.info(`Running in watch mode`);
  logger.info(`Watching for changes`);
  logger.info(`Ignoring changes in "**/{.git,node_modules}/**" | "${outDir}"`);
}

async function compilerHandler(callback: () => Promise<void>, config: Config) {
  const logger = new DefaultLogger();
  const startTime = performance.now();
  try {
    await callback();
  } catch (error) {
    logger.error(error);
  }
  const endTime = performance.now();
  const elapsedTime = Math.floor(endTime - startTime);
  logger.info(
    `⚡️ Build completed in ${chalk.green(
      `${elapsedTime}ms`
    )}! Resources emitted to ${chalk.green(config.config.output.path)}.`
  );
}

export async function restartServer(server: DevServer) {
  await server.close();
}
