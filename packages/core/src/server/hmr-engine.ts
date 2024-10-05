import fse from 'fs-extra';
// queue all updates and compile them one by one

import { stat } from 'node:fs/promises';
import { isAbsolute, relative } from 'node:path';

import type { Resource } from '@farmfe/runtime/src/resource-loader.js';
import { UserHmrConfig } from '../config/index.js';
import type { JsUpdateResult } from '../types/binding.js';
import { convertErrorMessage } from '../utils/error.js';
import { bold, formatExecutionTime, green, lightCyan } from '../utils/index.js';
import { WebSocketClient } from './ws.js';

export class HmrEngine {
  private _updateQueue: string[] = [];
  // private _updateResults: Map<string, { result: string; count: number }>;

  private _onUpdates: ((result: JsUpdateResult) => void)[];

  private _lastModifiedTimestamp: Map<string, string>;
  constructor(
    // compiler: Compiler,
    // devServer: HttpServer,
    // config: UserConfig,
    // ws: WebSocketServer,
    // private _logger: Logger
    private readonly app: any
  ) {
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
          return relative(this.app.compiler.config.config.root, item);
        } else {
          const resolvedPath = this.app.compiler.transformModulePath(
            this.app.compiler.config.config.root,
            item
          );
          return relative(this.app.compiler.config.config.root, resolvedPath);
        }
      })
      .join(', ');
    if (updatedFilesStr.length >= 100) {
      updatedFilesStr =
        updatedFilesStr.slice(0, 100) + `...(${queue.length} files)`;
    }

    // try {
    // we must add callback before update
    this.app.compiler.onUpdateFinish(async () => {
      // if there are more updates, recompile again
      if (this._updateQueue.length > 0) {
        await this.recompileAndSendResult();
      }
      if (this.app.resolvedUserConfig?.server.writeToDisk) {
        this.app.compiler.writeResourcesToDisk();
      }
    });

    const start = performance.now();

    const result = await this.app.compiler.update(queue);

    this.app.logger.info(
      `${bold(lightCyan(updatedFilesStr))} updated in ${bold(
        green(
          formatExecutionTime(
            performance.now() - start,
            this.app.resolvedUserConfig.timeUnit
          )
        )
      )}`,
      true
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

    this.app.ws.wss.clients.forEach((client: WebSocketClient) => {
      // @ts-ignore
      client.send(`
        {
          type: 'farm-update',
          result: ${resultStr}
        }
      `);
    });
    // } catch (err) {
    // checkClearScreen(this.app.compiler.config.config);
    // this.app.logger.error(convertErrorMessage(err));

    // }
  };

  async hmrUpdate(absPath: string | string[], force = false) {
    const paths = Array.isArray(absPath) ? absPath : [absPath];
    for (const path of paths) {
      if (
        this.app.compiler.hasModule(path) &&
        !this._updateQueue.includes(path)
      ) {
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

    if (!this.app.compiler.compiling && this._updateQueue.length > 0) {
      try {
        await this.recompileAndSendResult();
      } catch (e) {
        // eslint-disable-next-line no-control-regex
        const serialization = e.message.replace(/\x1b\[[0-9;]*m/g, '');
        const errorStr = `${JSON.stringify({
          message: serialization
        })}`;

        this.app.ws.wss.clients.forEach((client: WebSocketClient) => {
          // @ts-ignore
          // client.rawSend(`
          client.send(`
            {
              type: 'error',
              err: ${errorStr},
              overlay: ${
                (this.app.resolvedUserConfig.server.hmr as UserHmrConfig)
                  .overlay
              }
            }
          `);
        });

        this.app.logger.error(convertErrorMessage(e), true);
        // throw new Error(`hmr update failed: ${e.stack}`);
      }
    }
  }
}

function formatHmrResult(array: string[]) {
  return array.map((item) => `'${item.replaceAll('\\', '\\\\')}'`).join(', ');
}
