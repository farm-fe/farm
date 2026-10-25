import { stat } from 'node:fs/promises';
import { isAbsolute, relative } from 'node:path';
import type { Resource } from '@farmfe/runtime';
import fse from 'fs-extra';
import { HmrOptions } from '../config/index.js';
import type { CompilerUpdateItem, JsUpdateResult } from '../types/binding.js';
import { convertErrorMessage } from '../utils/error.js';
import { bold, cyan, green } from '../utils/index.js';
import { Server as FarmDevServer } from './index.js';

export class HmrEngine {
  private _updateQueue: CompilerUpdateItem[] = [];

  private _recompileInProgress = false;

  private _onUpdates!: ((result: JsUpdateResult) => void)[];

  private _lastModifiedTimestamp: Map<string, string>;
  constructor(private readonly devServer: FarmDevServer) {
    this._lastModifiedTimestamp = new Map();
  }

  callUpdates(result: JsUpdateResult) {
    this._onUpdates?.forEach((cb) => cb(result));
  }

  onUpdateFinish(cb: (result: JsUpdateResult) => void) {
    if (!this._onUpdates) {
      this._onUpdates = [];
    }
    this._onUpdates.push(cb);
  }

  recompileAndSendResult = async (): Promise<JsUpdateResult | undefined> => {
    if (this._recompileInProgress) return;
    const queue = [...this._updateQueue];

    if (queue.length === 0) {
      return;
    }
    const compiler = this.devServer.compiler;
    if (!compiler) return;
    // Completion callbacks can run before update() resolves. Consume this batch
    // first so those callbacks only see genuinely new filesystem events.
    this._updateQueue = this._updateQueue.filter(
      (item) => !queue.includes(item)
    );
    this._recompileInProgress = true;
    let result: JsUpdateResult;
    try {
      const compilerRoot = compiler.config.root ?? '';
      const logger = this.devServer.logger;
      let updatedFilesStr = queue
        .map((item) => {
          if (isAbsolute(item.path)) {
            return relative(compilerRoot, item.path);
          } else {
            const resolvedPath = compiler.transformModulePath(
              compilerRoot,
              item.path
            );
            return relative(compilerRoot, resolvedPath);
          }
        })
        .join(', ');
      if (updatedFilesStr.length >= 100) {
        updatedFilesStr =
          updatedFilesStr.slice(0, 100) + `...(${queue.length} files)`;
      }

      // we must add callback before update
      compiler.onUpdateFinish(async () => {
        // if there are more updates, recompile again
        if (!this._recompileInProgress && this._updateQueue.length > 0) {
          await this.recompileAndSendResult();
        }
        if (this.devServer.config?.server?.writeToDisk) {
          compiler.writeResourcesToDisk();
        }
      });

      const start = performance.now();

      result = await compiler.update(queue);

      logger.info(
        `${bold(cyan(updatedFilesStr))} updated in ${bold(green(logger.formatTime(performance.now() - start)))}`
      );

      this.sendUpdate(result);
    } catch (error) {
      this._updateQueue = [
        ...queue.filter(
          (item) =>
            !this._updateQueue.some((pending) => pending.path === item.path)
        ),
        ...this._updateQueue
      ];
      throw error;
    } finally {
      this._recompileInProgress = false;
    }
    if (!compiler.compiling && this._updateQueue.length > 0) {
      await this.recompileAndSendResult();
    }
    return result;
  };

  /** Broadcast an already completed update after its resources are published. */
  sendUpdate(result: JsUpdateResult): void {
    let dynamicResourcesMap: Record<string, Resource[]> | null = null;

    if (result.dynamicResourcesMap) {
      for (const [key, value] of Object.entries(result.dynamicResourcesMap)) {
        if (!dynamicResourcesMap) {
          dynamicResourcesMap = {} as Record<string, Resource[]>;
        }

        // @ts-ignore
        dynamicResourcesMap[key] = value.map((r) => ({
          path: r[0],
          type: r[1] as 'script' | 'link'
        }));
      }
    }
    const {
      added,
      changed,
      removed,
      immutableModules,
      mutableModules,
      boundaries
    } = result;
    const resultStr = `{
        added: [${formatHmrResult(added)}],
        changed: [${formatHmrResult(changed)}],
        removed: [${formatHmrResult(removed)}],
        immutableModules: ${JSON.stringify(immutableModules.trim())},
        mutableModules: ${JSON.stringify(mutableModules.trim())},
        boundaries: ${JSON.stringify(boundaries)},
        dynamicResourcesMap: ${JSON.stringify(dynamicResourcesMap)}
      }`;

    this.callUpdates(result);

    this.devServer.ws.wss.clients.forEach((client) => {
      client.send(`
        {
          type: 'farm-update',
          result: ${resultStr}
        }
      `);
    });
  }

  async hmrUpdate(
    absPath: CompilerUpdateItem | CompilerUpdateItem[],
    force = false
  ) {
    const compiler = this.devServer.compiler;
    if (!compiler) return;
    // Native compilation may hold the module graph while waiting for JS hooks.
    // Yield until it finishes before making synchronous graph queries.
    await compiler.waitForCompileFinish();
    const pathItems = Array.isArray(absPath) ? absPath : [absPath];
    for (const item of pathItems) {
      const queuedIndex = this._updateQueue.findIndex(
        (pending) => pending.path === item.path
      );
      if (
        item.type === 'added' ||
        compiler.hasModule(item.path) ||
        queuedIndex >= 0
      ) {
        if (fse.existsSync(item.path)) {
          const lastModifiedTimestamp = this._lastModifiedTimestamp.get(
            item.path
          );
          const currentTimestamp = (await stat(item.path)).mtime.toISOString();
          // only update the file if the timestamp changed since last update
          if (
            item.type === 'updated' &&
            !force &&
            lastModifiedTimestamp === currentTimestamp
          ) {
            continue;
          }
          this._lastModifiedTimestamp.set(item.path, currentTimestamp);
        }
        // push the path into the queue
        const currentQueuedIndex = this._updateQueue.findIndex(
          (pending) => pending.path === item.path
        );
        if (currentQueuedIndex >= 0) {
          // Membership events supersede earlier pending state for this path.
          if (item.type !== 'updated')
            this._updateQueue[currentQueuedIndex] = item;
        } else {
          this._updateQueue.push(item);
        }
      }
    }

    if (
      !compiler.compiling &&
      !this._recompileInProgress &&
      this._updateQueue.length > 0
    ) {
      try {
        await this.recompileAndSendResult();
      } catch (e: any) {
        const serialization = e.message.replace(/\x1b\[[0-9;]*m/g, '');
        const errorStr = `${JSON.stringify({
          message: serialization
        })}`;

        this.devServer.ws.wss.clients.forEach((client) => {
          client.send(`
            {
              type: 'error',
              err: ${errorStr},
              overlay: ${(this.devServer.config.server?.hmr as HmrOptions)?.overlay}
            }
          `);
        });

        this.devServer.logger.error(`Update Error: ${convertErrorMessage(e)}`);
      }
    }
  }
}

function formatHmrResult(array: string[]) {
  return array.map((item) => `'${item.replaceAll('\\', '\\\\')}'`).join(', ');
}
