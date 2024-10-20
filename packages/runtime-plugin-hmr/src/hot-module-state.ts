// Farm's HMR client is compatible with Vite, see https://vitejs.dev/guide/api-hmr.html.
// And it's inspired by both Vite and esm-hmr, see https://github.com/FredKSchott/esm-hmr
import { HmrClient } from './hmr-client';
import { logger } from './logger';

export class HotModuleState {
  acceptCallbacks: Array<{ deps: string[]; fn: (mods: any[]) => void }> = [];
  data = {};
  id: string;
  hmrClient: HmrClient;

  constructor(id: string, hmrClient: HmrClient) {
    this.id = id;
    this.hmrClient = hmrClient;
  }

  // the same as vite's hot.accept
  accept(deps?: any, callback?: (mods: any[]) => void) {
    if (typeof deps === 'function' || !deps) {
      // self-accept hot.accept(() => {})
      this.acceptCallbacks.push({
        deps: [this.id],
        fn: ([mod]) => deps?.(mod)
      });
    } else if (typeof deps === 'string') {
      // accept a single dependency hot.accept('./dep.js', () => {})
      this.acceptCallbacks.push({
        deps: [deps],
        fn: ([mod]) => callback?.(mod)
      });
    } else if (Array.isArray(deps)) {
      // accept multiple dependencies hot.accept(['./dep1.js', './dep2.js'], () => {})
      this.acceptCallbacks.push({ deps, fn: callback });
    } else {
      throw new Error('invalid hot.accept call');
    }
  }

  dispose(callback: (data: any) => void) {
    this.hmrClient.disposeMap.set(this.id, callback);
  }

  prune(callback: (data: any[]) => void) {
    this.hmrClient.pruneMap.set(this.id, callback);
  }

  acceptExports(
    _: string | readonly string[],
    _callback: (data: any) => void
  ): void {
    logger.debug('acceptExports is not supported for now');
  }

  decline() {
    /** does no thing */
  }

  invalidate(message?: string) {
    this.hmrClient.notifyListeners('vite:invalidate', {
      path: this.id,
      message
    });
    // notify the server to find the boundary starting from the parents of this module
    this.send('vite:invalidate', { path: this.id, message });
    this.send('farm:invalidate', { path: this.id, message });
    logger.debug(`invalidate ${this.id}${message ? `: ${message}` : ''}`);
  }

  on<T extends string>(event: T, cb: (payload: any) => void): void {
    const addToMap = (map: Map<string, any[]>) => {
      const existing = map.get(event) || [];
      existing.push(cb);
      map.set(event, existing);
    };
    addToMap(this.hmrClient.customListenersMap);
  }

  off<T extends string>(event: T, cb: (payload: any) => void): void {
    const removeFromMap = (map: Map<string, any[]>) => {
      const existing = map.get(event);
      if (existing === undefined) {
        return;
      }
      const pruned = existing.filter((l) => l !== cb);
      if (pruned.length === 0) {
        map.delete(event);
        return;
      }
      map.set(event, pruned);
    };
    removeFromMap(this.hmrClient.customListenersMap);
  }

  send<T extends string>(event: T, data?: any): void {
    if (this.hmrClient.socket.readyState === WebSocket.OPEN) {
      this.hmrClient.socket.send(
        JSON.stringify({ type: 'custom', event, data })
      );
    }
  }
}

export function createHotContext(id: string, hmrClient: HmrClient) {
  if (hmrClient.registeredHotModulesMap.has(id)) {
    const hotModuleState = hmrClient.registeredHotModulesMap.get(id);
    hotModuleState.acceptCallbacks = []; // clear the accept callbacks when hot reloading
    return hotModuleState;
  }

  const state = new HotModuleState(id, hmrClient);
  hmrClient.registeredHotModulesMap.set(id, state);
  return state;
}
