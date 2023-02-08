// queue all updates and compile them one by one

import { Compiler } from '../compiler/index.js';
import { DevServer } from './index.js';
import debounce from 'lodash.debounce';

export class HmrEngine {
  private _updateQueue = new Set<string>();
  private _updateResults: Map<string, string> = new Map();

  private _compiler: Compiler;
  private _devServer: DevServer;

  constructor(compiler: Compiler, devServer: DevServer) {
    this._compiler = compiler;
    this._devServer = devServer;
  }

  recompileAndSendResult = debounce(async (): Promise<void> => {
    const queue = [...this._updateQueue];
    this._updateQueue = new Set();
    const result = await this._compiler.update(queue);

    // TODO auto detect the boundary
    const resultStr = `export default {
      added: [${result.added.map((r) => `'${r}'`).join(', ')}],
      changed: [${result.changed.map((r) => `'${r}'`).join(', ')}],
      removed: [${result.removed.map((r) => `'${r}'`).join(', ')}],
      modules: ${result.modules.trim().slice(0, -1)},
      boundaries: ${`{ ${result.changed
        .map((r) => `'${r}': ['${r}']`)
        .join(', ')} }`}
    }`;

    console.log(result, resultStr);

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
    this._updateQueue.add(path);
    return this.recompileAndSendResult();
  }

  getUpdateResult(id: string) {
    const result = this._updateResults.get(id);
    this._updateResults.delete(id);
    return result;
  }
}
