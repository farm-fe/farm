// queue all updates and compile them one by one

import { isAbsolute, relative } from 'node:path';

import { Compiler } from '../compiler/index.js';
import { DevServer } from './index.js';
import { Logger, bold, cyan, green } from '../utils/index.js';
import { JsUpdateResult } from '../../binding/binding.js';
import type { Resource } from '@farmfe/runtime/src/resource-loader.js';

export class HmrEngine {
  private _updateQueue: string[] = [];
  // private _updateResults: Map<string, { result: string; count: number }> =
  //   new Map();

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

    this._updateQueue = [];
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
    this._logger.info(
      `${cyan(updatedFilesStr)} updated in ${bold(
        green(`${Date.now() - start}ms`)
      )}`
    );

    // TODO: write resources to disk when hmr finished in incremental mode
    // if (this._devServer.config?.writeToDisk) {
    //   this._compiler.onUpdateFinish(() => {
    //     this._compiler.writeResourcesToDisk();
    //     console.log('writeResourcesToDisk');
    //   });
    // }

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
      modules: ${
        result.modules.trim().endsWith(';')
          ? result.modules.trim().slice(0, -1)
          : result.modules.trim()
      },
      boundaries: ${JSON.stringify(result.boundaries)},
      dynamicResourcesMap: ${JSON.stringify(dynamicResourcesMap)}
    }`;

    this.callUpdates(result);

    // const id = Date.now().toString();
    // // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // // @ts-ignore TODO fix this
    // this._updateResults.set(id, {
    //   result: resultStr,
    //   count: this._devServer.ws.clients.size
    // });
    this._devServer.ws.clients.forEach((client: any) => {
      client.send(resultStr);
    });

    // if there are more updates, recompile again
    if (this._updateQueue.length > 0) {
      await this.recompileAndSendResult();
    }
  };

  async hmrUpdate(path: string) {
    // if lazy compilation is enabled, we need to update the virtual module
    if (this._compiler.config.config.lazyCompilation) {
      if (this._compiler.hasModule(path) && !this._updateQueue.includes(path)) {
        this._updateQueue.push(path);
      }

      if (!this._compiler.compiling) {
        await this.recompileAndSendResult();
      }
    } else if (this._compiler.hasModule(path)) {
      if (!this._updateQueue.includes(path)) {
        this._updateQueue.push(path);
      }

      if (!this._compiler.compiling) {
        await this.recompileAndSendResult();
      }
    }
  }

  // getUpdateResult(id: string) {
  //   const result = this._updateResults.get(id);

  //   if (result) {
  //     result.count--;

  //     // there are no more clients waiting for this update
  //     if (result.count <= 0 && this._updateResults.size >= 2) {
  //       /**
  //        * Edge handle
  //        * The BrowserExtension the user's browser may replay the request, resulting in an error that the result.id cannot be found.
  //        * So keep the result of the last time every time, so that the request can be successfully carried out.
  //        */
  //       this._updateResults.delete(this._updateResults.keys().next().value);
  //     }
  //   }
  //   return result?.result;
  // }
}
