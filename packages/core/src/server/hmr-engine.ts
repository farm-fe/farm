// queue all updates and compile them one by one

import { Compiler } from '../compiler/index.js';
import { DevServer } from './index.js';
import debounce from 'lodash.debounce';
import { Logger } from '../logger.js';
import { relative } from 'path';
import chalk from 'chalk';

export class HmrEngine {
  private _updateQueue = new Set<string>();
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
    this._updateQueue = new Set();
    let updatedFilesStr = queue
      .map((item) => relative(this._compiler.config.config.root, item))
      .join(', ');
    if (updatedFilesStr.length >= 100) {
      updatedFilesStr =
        updatedFilesStr.slice(0, 100) + `(${queue.length} files})`;
    }

    const start = Date.now();
    const result = await this._compiler.update(queue);
    this._logger.info(
      `${chalk.cyan(updatedFilesStr)} updated in ${chalk.green.bold(
        `${Date.now() - start}ms`
      )}`
    );

    // TODO auto detect the boundary
    const resultStr = `export default {
      added: [${result.added.map((r) => `'${r}'`).join(', ')}],
      changed: [${result.changed.map((r) => `'${r}'`).join(', ')}],
      removed: [${result.removed.map((r) => `'${r}'`).join(', ')}],
      modules: ${result.modules.trim().slice(0, -1)},
      boundaries: ${JSON.stringify(result.boundaries)},
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
  }, 200);

  async hmrUpdate(path: string) {
    if (!this._compiler.compiling && this._compiler.hasModule(path)) {
      this._updateQueue.add(path);
      await this.recompileAndSendResult();
    }
  }

  getUpdateResult(id: string) {
    const result = this._updateResults.get(id);
    this._updateResults.delete(id);
    return result;
  }
}
