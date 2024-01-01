// queue all updates and compile them one by one

import { isAbsolute, relative } from 'node:path';

import { Compiler } from '../compiler/index.js';
import { DevServer } from './index.js';
import { Logger, bold, clearScreen, cyan, green } from '../utils/index.js';
import { JsUpdateResult } from '../../binding/binding.js';
import type { Resource } from '@farmfe/runtime/src/resource-loader.js';
import { WebSocketClient } from './ws.js';

export class HmrEngine {
  private _updateQueue: string[] = [];
  // private _updateResults: Map<string, { result: string; count: number }> =

  private _compiler: Compiler;
  private _devServer: DevServer;
  private _onUpdates: ((result: JsUpdateResult) => void)[];

  constructor(
    compiler: Compiler,
    devServer: DevServer,
    private _logger: Logger
  ) {
    this._compiler = compiler;
    this._devServer = devServer;
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

    const start = Date.now();

    const result = await this._compiler.update(queue);
    clearScreen();
    this._logger.info(
      `${cyan(updatedFilesStr)} updated in ${bold(
        green(`${Date.now() - start}ms`)
      )}`
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
        dynamicResourcesMap[key] = value.map((r) => ({
          path: r[0],
          type: r[1] as 'script' | 'link'
        }));
      }
    }

    const resultStr = `{
      added: [${result.added
        .map((r) => `'${r.replaceAll('\\', '\\\\')}'`)
        .join(', ')}],
      changed: [${result.changed
        .map((r) => `'${r.replaceAll('\\', '\\\\')}'`)
        .join(', ')}],
      removed: [${result.removed
        .map((r) => `'${r.replaceAll('\\', '\\\\')}'`)
        .join(', ')}],
      immutableModules: ${JSON.stringify(result.immutableModules.trim())},
      mutableModules: ${JSON.stringify(result.mutableModules.trim())},
      boundaries: ${JSON.stringify(result.boundaries)},
      dynamicResourcesMap: ${JSON.stringify(dynamicResourcesMap)}
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

    this._compiler.onUpdateFinish(async () => {
      // if there are more updates, recompile again
      if (this._updateQueue.length > 0) {
        await this.recompileAndSendResult();
      }
    });
  };

  async hmrUpdate(absPath: string | string[]) {
    const paths = Array.isArray(absPath) ? absPath : [absPath];

    for (const path of paths) {
      if (this._compiler.hasModule(path) && !this._updateQueue.includes(path)) {
        this._updateQueue.push(path);
      }
    }

    if (!this._compiler.compiling) {
      try {
        await this.recompileAndSendResult();
      } catch (e) {
        // eslint-disable-next-line no-control-regex
        const serialization = e.message.replace(/\x1b\[[0-9;]*m/g, "");
        const errorStr = `${JSON.stringify({
          message: serialization
        })}`;
        this._devServer.ws.clients.forEach((client: WebSocketClient) => {
          client.rawSend(`
            {
              type: 'error',
              result: ${errorStr}
            }
          `);
        });
        this._logger.error(e);
      }
    }
  }
}


