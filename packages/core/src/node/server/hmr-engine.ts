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
    const result = await this._compiler.update([...this._updateQueue]);
    this._updateQueue = new Set();

    const id = Date.now().toString();
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore TODO fix this
    this._updateResults.set(id, result);

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
