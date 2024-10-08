import fse from 'fs-extra';
// queue all updates and compile them one by one

import { stat } from 'node:fs/promises';
import { isAbsolute, relative } from 'node:path';

import { Compiler } from '../compiler/index.js';
import { checkClearScreen } from '../config/index.js';
import type { JsUpdateResult } from '../types/binding.js';
import {
  Logger,
  bold,
  cyan,
  getDynamicResources,
  green
} from '../utils/index.js';
import { logError } from './error.js';
import { Server } from './index.js';
import { WebSocketClient } from './ws.js';

export class HmrEngine {
  private _updateQueue: string[] = [];
  // private _updateResults: Map<string, { result: string; count: number }> =

  private _compiler: Compiler;
  private _devServer: Server;
  private _onUpdates: ((result: JsUpdateResult) => void)[];

  private _lastModifiedTimestamp: Map<string, string>;

  constructor(
    compiler: Compiler,
    devServer: Server,
    private _logger: Logger
  ) {
    this._compiler = compiler;
    this._devServer = devServer;
    // this._lastAttemptWasError = false;
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

    let updatedFilesStr = queue
      .map((item) => {
        if (isAbsolute(item)) {
          return relative(this._compiler.config.config.root, item);
        } else {
          const resolvedPath = this._compiler.transformModulePath(
            this._compiler.config.config.root,
            item
          );
          return relative(this._compiler.config.config.root, resolvedPath);
        }
      })
      .join(', ');
    if (updatedFilesStr.length >= 100) {
      updatedFilesStr =
        updatedFilesStr.slice(0, 100) + `...(${queue.length} files)`;
    }

    try {
      // we must add callback before update
      this._compiler.onUpdateFinish(async () => {
        // if there are more updates, recompile again
        if (this._updateQueue.length > 0) {
          await this.recompileAndSendResult();
        }
        if (this._devServer?.config?.writeToDisk) {
          this._compiler.writeResourcesToDisk();
        }
      });

      checkClearScreen(this._compiler.config.config);
      const start = Date.now();
      const result = await this._compiler.update(queue);
      this._logger.info(
        `${bold(cyan(updatedFilesStr))} updated in ${bold(
          green(`${Date.now() - start}ms`)
        )}`
      );

      // clear update queue after update finished
      this._updateQueue = this._updateQueue.filter(
        (item) => !queue.includes(item)
      );

      const { dynamicResources, dynamicModuleResourcesMap } =
        getDynamicResources(result.dynamicResourcesMap);

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
        dynamicResources: ${JSON.stringify(dynamicResources)},
        dynamicModuleResourcesMap: ${JSON.stringify(dynamicModuleResourcesMap)}
      }`;

      this.callUpdates(result);

      this._devServer.ws.clients.forEach((client: WebSocketClient) => {
        client.rawSend(`
        {
          type: 'farm-update',
          result: ${resultStr}
        }
      `);
      });
    } catch (err) {
      checkClearScreen(this._compiler.config.config);
      throw new Error(logError(err) as unknown as string);
    }
  };

  async hmrUpdate(absPath: string | string[], force = false) {
    const paths = Array.isArray(absPath) ? absPath : [absPath];

    for (const path of paths) {
      if (this._compiler.hasModule(path) && !this._updateQueue.includes(path)) {
        if (fse.existsSync(path)) {
          const lastModifiedTimestamp = this._lastModifiedTimestamp.get(path);
          const currentTimestamp = (await stat(path)).mtime.toISOString();
          // only update the file if the timestamp changed since last update
          if (!force && lastModifiedTimestamp === currentTimestamp) {
            continue;
          }
          this._lastModifiedTimestamp.set(path, currentTimestamp);
        }
        // push the path into the queue
        this._updateQueue.push(path);
      }
    }

    if (!this._compiler.compiling && this._updateQueue.length > 0) {
      try {
        await this.recompileAndSendResult();
      } catch (e) {
        // eslint-disable-next-line no-control-regex
        const serialization = e.message.replace(/\x1b\[[0-9;]*m/g, '');
        const errorStr = `${JSON.stringify({
          message: serialization
        })}`;
        this._devServer.ws.clients.forEach((client: WebSocketClient) => {
          client.rawSend(`
            {
              type: 'error',
              err: ${errorStr},
              overlay: ${this._devServer.config.hmr.overlay}
            }
          `);
        });
        this._logger.error(e);
      }
    }
  }
}

function formatHmrResult(array: string[]) {
  return array.map((item) => `'${item.replaceAll('\\', '\\\\')}'`).join(', ');
}
