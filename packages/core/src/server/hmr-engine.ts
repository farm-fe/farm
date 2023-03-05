// queue all updates and compile them one by one

import { Compiler } from '../compiler/index.js';
import { DevServer } from './index.js';
import debounce from 'lodash.debounce';
import { Logger } from '../logger.js';
import { relative } from 'path';
import chalk from 'chalk';
import type { Resource } from '@farmfe/runtime/src/resource-loader.js';

export class HmrEngine {
  private _updateQueue: string[] = [];
  private _updateResults: Map<string, string> = new Map();

  private _compiler: Compiler;
  private _devServer: DevServer;

  constructor(
    compiler: Compiler,
    devServer: DevServer,
    private _logger: Logger
  ) {
    this._compiler = compiler;
    this._devServer = devServer;
  }

  recompileAndSendResult = debounce(async (): Promise<void> => {
    const queue = [...this._updateQueue];

    if (queue.length === 0) {
      return;
    }

    this._updateQueue = [];
    let updatedFilesStr = queue
      .map((item) => relative(this._compiler.config.config.root, item))
      .join(', ');
    if (updatedFilesStr.length >= 100) {
      updatedFilesStr =
        updatedFilesStr.slice(0, 100) + `...(${queue.length} files)`;
    }

    const start = Date.now();
    const result = await this._compiler.update(queue);
    this._logger.info(
      `${chalk.cyan(updatedFilesStr)} updated in ${chalk.green.bold(
        `${Date.now() - start}ms`
      )}`
    );
    let dynamicResourcesMap: Record<string, Resource[]> = null;

    if (result.dynamicResourcesMap) {
      for (const [key, value] of Object.entries(result.dynamicResourcesMap)) {
        if (!dynamicResourcesMap) {
          dynamicResourcesMap = {} as Record<string, Resource[]>;
        }
        dynamicResourcesMap[key] = value.map((r) => ({
          path: r[0],
          type: r[1] as 'script' | 'link',
        }));
      }
    }

    const resultStr = `export default {
      added: [${result.added.map((r) => `'${r}'`).join(', ')}],
      changed: [${result.changed.map((r) => `'${r}'`).join(', ')}],
      removed: [${result.removed.map((r) => `'${r}'`).join(', ')}],
      modules: ${result.modules.trim().slice(0, -1)},
      boundaries: ${JSON.stringify(result.boundaries)},
      dynamicResourcesMap: ${JSON.stringify(dynamicResourcesMap)}
    }`;

    const id = Date.now().toString();
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore TODO fix this
    this._updateResults.set(id, resultStr);

    this._devServer.ws.clients.forEach((client) => {
      client.send(
        JSON.stringify({
          id,
        })
      );
    });

    // if there are more updates, recompile again
    if (this._updateQueue.length > 0) {
      await this.recompileAndSendResult();
    }
  }, 200);

  async hmrUpdate(path: string) {
    // if lazy compilation is enabled, we need to update the virtual module
    if (this._compiler.config.config.lazyCompilation) {
      const lazyCompiledModule = `virtual:FARMFE_DYNAMIC_IMPORT:${path}`;

      if (
        this._compiler.hasModule(lazyCompiledModule) &&
        !this._updateQueue.includes(lazyCompiledModule)
      ) {
        this._updateQueue.push(lazyCompiledModule);
      }

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

  getUpdateResult(id: string) {
    const result = this._updateResults.get(id);
    this._updateResults.delete(id);
    return result;
  }
}
