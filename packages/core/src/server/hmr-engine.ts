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

  private _onUpdates: ((result: JsUpdateResult) => void)[];

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

  recompileAndSendResult = async (): Promise<JsUpdateResult> => {
    const queue = [...this._updateQueue];

    if (queue.length === 0) {
      return;
    }
    const logger = this.devServer.logger;
    let updatedFilesStr = queue
      .map((item) => {
        if (isAbsolute(item.path)) {
          return relative(this.devServer.compiler.config.root, item.path);
        } else {
          const resolvedPath = this.devServer.compiler.transformModulePath(
            this.devServer.compiler.config.root,
            item.path
          );
          return relative(this.devServer.compiler.config.root, resolvedPath);
        }
      })
      .join(', ');
    if (updatedFilesStr.length >= 100) {
      updatedFilesStr =
        updatedFilesStr.slice(0, 100) + `...(${queue.length} files)`;
    }

    // we must add callback before update
    this.devServer.compiler.onUpdateFinish(async () => {
      // if there are more updates, recompile again
      if (this._updateQueue.length > 0) {
        await this.recompileAndSendResult();
      }
      if (this.devServer.config?.server.writeToDisk) {
        this.devServer.compiler.writeResourcesToDisk();
      }
    });

    const start = performance.now();

    const result = await this.devServer.compiler.update(queue);

    logger.info(
      `${bold(cyan(updatedFilesStr))} updated in ${bold(green(logger.formatTime(performance.now() - start)))}`
    );

    // clear update queue after update finished
    this._updateQueue = this._updateQueue.filter(
      (item) => !queue.includes(item)
    );

    let dynamicResourcesMap: Record<string, Resource[]> = null;

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
    // TODO optimize this part
    // } catch (err) {
    // checkClearScreen(this.devServer.compiler.config.config);
    // this.devServer.logger.error(convertErrorMessage(err));

    // }
  };

  async hmrUpdate(
    absPath: CompilerUpdateItem | CompilerUpdateItem[],
    force = false
  ) {
    const pathItems = Array.isArray(absPath) ? absPath : [absPath];
    for (const item of pathItems) {
      if (
        this.devServer.compiler.hasModule(item.path) &&
        !this._updateQueue.find((queueItem) => queueItem.path === item.path)
      ) {
        if (fse.existsSync(item.path)) {
          const lastModifiedTimestamp = this._lastModifiedTimestamp.get(
            item.path
          );
          const currentTimestamp = (await stat(item.path)).mtime.toISOString();
          // only update the file if the timestamp changed since last update
          if (!force && lastModifiedTimestamp === currentTimestamp) {
            continue;
          }
          this._lastModifiedTimestamp.set(item.path, currentTimestamp);
        }
        // push the path into the queue
        this._updateQueue.push(item);
      }
    }

    if (!this.devServer.compiler.compiling && this._updateQueue.length > 0) {
      try {
        await this.recompileAndSendResult();
      } catch (e) {
        const serialization = e.message.replace(/\x1b\[[0-9;]*m/g, '');
        const errorStr = `${JSON.stringify({
          message: serialization
        })}`;

        this.devServer.ws.wss.clients.forEach((client) => {
          // @ts-ignore
          // client.rawSend(`
          client.send(`
            {
              type: 'error',
              err: ${errorStr},
              overlay: ${(this.devServer.config.server.hmr as HmrOptions).overlay}
            }
          `);
        });

        this.devServer.logger.error(convertErrorMessage(e), {
          exit: true
        });
        // throw new Error(`hmr update failed: ${e.stack}`);
      }
    }
  }
}

function formatHmrResult(array: string[]) {
  return array.map((item) => `'${item.replaceAll('\\', '\\\\')}'`).join(', ');
}
