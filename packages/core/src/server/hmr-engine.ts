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
    const queue = [...this._updateQueue];

    if (queue.length === 0) {
      return;
    }
    const compiler = this.devServer.compiler;
    if (!compiler) return;
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
      if (this._updateQueue.length > 0) {
        await this.recompileAndSendResult();
      }
      if (this.devServer.config?.server?.writeToDisk) {
        compiler.writeResourcesToDisk();
      }
    });

    const start = performance.now();

    const result = await compiler.update(queue);

    logger.info(
      `${bold(cyan(updatedFilesStr))} updated in ${bold(green(logger.formatTime(performance.now() - start)))}`
    );

    // clear update queue after update finished
    this._updateQueue = this._updateQueue.filter(
      (item) => !queue.includes(item)
    );

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
  };

  async hmrUpdate(
    absPath: CompilerUpdateItem | CompilerUpdateItem[],
    force = false
  ) {
    const compiler = this.devServer.compiler;
    if (!compiler) return;
    const pathItems = Array.isArray(absPath) ? absPath : [absPath];
    for (const item of pathItems) {
      if (
        compiler.hasModule(item.path) &&
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

    if (!compiler.compiling && this._updateQueue.length > 0) {
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
