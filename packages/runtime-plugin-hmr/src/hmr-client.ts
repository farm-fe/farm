import type { ModuleSystem } from '@farmfe/runtime';
import { HMRPayload, HmrUpdateResult, RawHmrUpdateResult } from './types';
import { HotModuleState } from './hot-module-state';
import { logger } from './logger';

// Inject during compile time
const port = Number(FARM_HMR_PORT || 9000);
// TODO use import.meta to get hostname
const host =
  typeof FARM_HMR_HOST === 'boolean'
    ? window.location.hostname || 'localhost'
    : FARM_HMR_HOST || 'localhost';

const path = FARM_HMR_PATH || '/__hmr';

export class HmrClient {
  socket: WebSocket;
  registeredHotModulesMap = new Map<string, HotModuleState>();
  disposeMap = new Map<string, (data: any) => void | Promise<void>>();
  pruneMap = new Map<string, (data: any) => void | Promise<void>>();
  customListenersMap = new Map<
    string,
    ((data: any) => void | Promise<void>)[]
  >();

  constructor(private moduleSystem: ModuleSystem) {}

  connect() {
    logger.log('connecting to the server...');

    // setup websocket connection
    const socket = new WebSocket(`ws://${host}:${port}${path}`, 'farm_hmr');
    this.socket = socket;
    // listen for the message from the server
    // when the user save the file, the server will recompile the file(and its dependencies as long as its dependencies are changed)
    // after the file is recompiled, the server will generated a update resource and send its id to the client
    // the client will apply the update
    socket.addEventListener('message', (event) => {
      const result: HMRPayload = eval(`(${event.data})`);
      this.handleMessage(result);
    });

    socket.addEventListener(
      'open',
      () => {
        this.notifyListeners('vite:ws:connect', { webSocket: socket });
      },
      { once: true }
    );

    socket.addEventListener('close', () => {
      this.notifyListeners('vite:ws:disconnect', { webSocket: socket });
      logger.log('disconnected from the server, please reload the page.');
    });

    return socket;
  }

  async applyHotUpdates(result: HmrUpdateResult, moduleSystem: ModuleSystem) {
    result.changed.forEach((id) => {
      logger.log(`${id} updated`);
    });

    for (const id of result.removed) {
      const prune = this.pruneMap.get(id);
      if (prune) {
        const hotContext = this.registeredHotModulesMap.get(id);
        await prune(hotContext.data);
      }

      moduleSystem.delete(id);
      this.registeredHotModulesMap.delete(id);
    }

    for (const id of result.added) {
      moduleSystem.register(id, result.modules[id]);
    }

    for (const id of result.changed) {
      moduleSystem.update(id, result.modules[id]);

      if (!result.boundaries[id]) {
        // do not found boundary module, reload the window
        window.location.reload();
      }
    }

    if (result.dynamicResourcesMap) {
      moduleSystem.dynamicModuleResourcesMap = result.dynamicResourcesMap;
    }

    for (const chains of Object.values(result.boundaries)) {
      for (const chain of chains) {
        // clear the cache of the boundary module and its dependencies
        for (const id of chain) {
          moduleSystem.clearCache(id);
        }

        try {
          // require the boundary module
          const boundary = chain[chain.length - 1];
          const hotContext = this.registeredHotModulesMap.get(boundary);
          const acceptedDep =
            chain.length > 1 ? chain[chain.length - 2] : undefined;

          if (!hotContext) {
            console.error('hot context is empty for ', boundary);
            window.location.reload();
          }

          // get all the accept callbacks of the boundary module that accepts the updated module
          const selfAcceptedCallbacks = hotContext.acceptCallbacks.filter(
            ({ deps }) => deps.includes(boundary)
          );
          const depsAcceptedCallbacks = hotContext.acceptCallbacks.filter(
            ({ deps }) => deps.includes(acceptedDep)
          );
          // when there are both self accept callbacks and deps accept callbacks in a boundary module, only the deps accept callbacks will be called
          for (const [acceptedId, acceptedCallbacks] of Object.entries({
            [acceptedDep]: depsAcceptedCallbacks,
            [boundary]: selfAcceptedCallbacks
          })) {
            if (acceptedCallbacks.length > 0) {
              const acceptHotContext =
                this.registeredHotModulesMap.get(acceptedId);

              const disposer = this.disposeMap.get(acceptedId);
              if (disposer) await disposer(acceptHotContext.data);
              // clear accept callbacks, it will be re-registered in the accepted module when the module is required
              acceptHotContext.acceptCallbacks = [];

              const acceptedExports = moduleSystem.require(acceptedId);

              for (const { deps, fn } of acceptedCallbacks) {
                fn(
                  deps.map((dep) =>
                    dep === acceptedId ? acceptedExports : undefined
                  )
                );
              }
              // break the loop, only the first accept callback will be called
              break;
            }
          }
        } catch (err) {
          // The boundary module's dependencies may not present in current module system for a multi-page application. We should reload the window in this case.
          // See https://github.com/farm-fe/farm/issues/383
          console.error(err);
          window.location.reload();
        }
      }
    }
  }

  async notifyListeners(event: string, data: any) {
    const callbacks = this.customListenersMap.get(event);

    if (callbacks) {
      await Promise.allSettled(callbacks.map((cb) => cb(data)));
    }
  }

  /**
   * handle vite HMR message, except farm-update which is handled by handleFarmUpdate, other messages are handled the same as vite
   * @param payload Vite HMR payload
   */
  async handleMessage(payload: HMRPayload) {
    switch (payload.type) {
      case 'farm-update':
        this.notifyListeners('farm:beforeUpdate', payload);
        this.handleFarmUpdate(payload.result);
        this.notifyListeners('farm:afterUpdate', payload);
        break;
      case 'connected':
        logger.log('connected to the server');
        break;
      case 'update':
        this.notifyListeners('vite:beforeUpdate', payload);
        await Promise.all(
          payload.updates.map(async (update) => {
            if (update.type === 'js-update') {
              this.socket.send(JSON.stringify(update));
              return;
            }

            logger.warn('css link update is not supported yet');
          })
        );
        this.notifyListeners('vite:afterUpdate', payload);
        break;
      case 'custom':
        this.notifyListeners(payload.event, payload.data);
        break;
      case 'full-reload':
        this.notifyListeners('vite:beforeFullReload', payload);
        window.location.reload();
        break;
      case 'prune':
        this.notifyListeners('vite:beforePrune', payload);
        break;
      case 'error':
        this.notifyListeners('vite:error', payload);
        // TODO support error overlay
        break;
      default:
        logger.warn(`unknown message payload: ${payload}`);
    }
  }

  handleFarmUpdate(result: RawHmrUpdateResult) {
    const immutableModules = eval(result.immutableModules);
    const mutableModules = eval(result.mutableModules);
    const modules = { ...immutableModules, ...mutableModules };
    this.applyHotUpdates(
      {
        added: result.added,
        changed: result.changed,
        removed: result.removed,
        boundaries: result.boundaries,
        modules,
        dynamicResourcesMap: result.dynamicResourcesMap
      },
      this.moduleSystem
    );
  }
}
