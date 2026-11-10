import { stat } from 'node:fs/promises';
import { isAbsolute, relative } from 'node:path';
import type { Resource } from '@farmfe/runtime';
import fse from 'fs-extra';
import { HmrOptions } from '../config/index.js';
import type { CompilerUpdateItem, JsUpdateResult } from '../types/binding.js';
import { convertErrorMessage } from '../utils/error.js';
import { bold, cyan, green, normalizePath } from '../utils/index.js';
import { Server as FarmDevServer } from './index.js';

interface PendingWatchUpdate {
  item: CompilerUpdateItem;
  captureImporters: boolean;
  resolve: () => void;
  reject: (error: unknown) => void;
}

export class HmrEngine {
  private _pendingWatchUpdates: PendingWatchUpdate[] = [];
  private _watchDrainRunning = false;

  private _updateQueue: CompilerUpdateItem[] = [];

  private _recompileInProgress = false;
  private _updateAttempt = 0;

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
    this._updateAttempt++;
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
        // Submission and the successful tail below own pending work. A delayed
        // listener must not independently retry work admitted by another owner.
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
    if (this._updateQueue.length > 0) {
      if (!compiler.compiling) {
        await this.recompileAndSendResult();
      } else {
        // Readiness is released by success, failed updates and full compilation.
        // Unlike onUpdateFinish it is not limited to successful update callbacks.
        // Attempt the pending work once; a failed HMR attempt does not reach this
        // successful tail and therefore cannot schedule its own automatic retry.
        void this.submitHmrUpdate(this._updateAttempt).catch((error) => {
          this.devServer.logger.error(
            `HMR pending submission failed: ${error}`
          );
        });
      }
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

  /** Watcher-only admission. Public/plugin hmrUpdate keeps its existing contract. */
  queueWatchUpdate(item: CompilerUpdateItem): Promise<void> {
    return this.enqueueWatchUpdate(item, false);
  }

  queueWatchRemove(path: string): Promise<void> {
    return this.enqueueWatchUpdate({ path, type: 'removed' }, true);
  }

  private enqueueWatchUpdate(
    item: CompilerUpdateItem,
    captureImporters: boolean
  ): Promise<void> {
    if (!this.devServer.compiler) return Promise.resolve();
    const promise = new Promise<void>((resolve, reject) => {
      this._pendingWatchUpdates.push({
        item: { ...item },
        captureImporters,
        resolve,
        reject
      });
    });
    if (!this._watchDrainRunning) void this.drainWatchUpdates();
    return promise;
  }

  private async drainWatchUpdates(): Promise<void> {
    if (this._watchDrainRunning) return;
    this._watchDrainRunning = true;
    try {
      while (this._pendingWatchUpdates.length) {
        const compiler = this.devServer.compiler;
        if (!compiler) {
          for (const request of this._pendingWatchUpdates.splice(0))
            request.resolve();
          return;
        }
        await compiler.waitForCompileFinish();
        const first = this._pendingWatchUpdates.shift();
        if (!first) continue;
        const batch = [first];
        // Only consecutive ordinary changes are coalesced. Membership admission
        // stays ordered; the compiler's retained failure queue is not isolated.
        if (batch[0].item.type === 'updated') {
          while (this._pendingWatchUpdates[0]?.item.type === 'updated') {
            const next = this._pendingWatchUpdates.shift();
            if (next) batch.push(next);
          }
        }
        if (batch[0].item.type === 'updated') {
          const prepared: PendingWatchUpdate[] = [];
          for (const request of batch) {
            try {
              // Independent watcher requests must not inherit the public array
              // API's fail-fast preparation semantics.
              await this.prepareHmrUpdate([request.item], false);
              prepared.push(request);
            } catch (error) {
              request.reject(error);
            }
          }
          if (prepared.length) {
            try {
              await this.submitHmrUpdate();
              for (const request of prepared) request.resolve();
            } catch (error) {
              for (const request of prepared) request.reject(error);
            }
          }
          continue;
        }
        try {
          const paths = batch.map((request) => request.item);
          if (batch[0].captureImporters) {
            // A resolved readiness Promise does not reserve the graph.
            while (compiler.compiling) await compiler.waitForCompileFinish();
            const file = paths[0].path;
            const parents = compiler.hasModule(file)
              ? compiler.getParentFiles(file)
              : [];
            paths.push(
              ...parents.map((path) => ({
                path: normalizePath(path),
                type: 'updated' as const
              }))
            );
          }
          await this.hmrUpdate(paths, batch[0].captureImporters);
          for (const request of batch) request.resolve();
        } catch (error) {
          for (const request of batch) request.reject(error);
        }
      }
    } catch (error) {
      for (const request of this._pendingWatchUpdates.splice(0))
        request.reject(error);
    } finally {
      this._watchDrainRunning = false;
    }
  }

  async hmrUpdate(
    absPath: CompilerUpdateItem | CompilerUpdateItem[],
    force = false
  ) {
    // Keep the public array API fail-fast: no submission after preparation rejects.
    await this.prepareHmrUpdate(
      Array.isArray(absPath) ? absPath : [absPath],
      force
    );
    await this.submitHmrUpdate();
  }

  private async prepareHmrUpdate(
    pathItems: CompilerUpdateItem[],
    force: boolean
  ) {
    const compiler = this.devServer.compiler;
    if (!compiler) return;
    // Native compilation may hold the module graph while waiting for JS hooks.
    // Yield until it finishes before making synchronous graph queries.
    await compiler.waitForCompileFinish();
    for (const item of pathItems) {
      // A preceding item's stat may have yielded to a lazy/manual update.
      // Do not yield between the final check and synchronous graph queries.
      while (compiler.compiling) await compiler.waitForCompileFinish();
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
  }

  private async submitHmrUpdate(expectedAttempt?: number) {
    const compiler = this.devServer.compiler;
    if (!compiler) return;
    // Only automatic tail wakes carry a token. A different callback may have
    // attempted this work while readiness was pending, including a failed
    // attempt: that transfers ownership and invalidates this stale wake.
    // Explicit public/watcher submissions retain their normal retry authority.
    const staleWake = () =>
      expectedAttempt !== undefined && expectedAttempt !== this._updateAttempt;
    if (staleWake()) return;
    // An active HMR recompile already owns the completion callback. Preserve
    // admission-only return for reentrant callers instead of awaiting ourselves.
    if (this._recompileInProgress) return;
    // Preparation yields. An unrelated lazy/manual compile can start in that
    // gap without installing an HMR wake-up callback; retain responsibility here.
    while (compiler.compiling) {
      await compiler.waitForCompileFinish();
      if (staleWake()) return;
      if (this._recompileInProgress) return;
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
