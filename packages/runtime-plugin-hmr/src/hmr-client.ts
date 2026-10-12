import type { ModuleSystem } from '@farmfe/runtime';
import type { HotModuleState } from './hot-module-state.js';
import { logger } from './logger.js';
import { ErrorOverlay, overlayId } from './overlay.js';
import type {
  HMRPayload,
  HmrUpdateResult,
  RawHmrUpdateResult
} from './types.js';

// Inject during compile time
const usingClientHost = typeof FARM_HMR_HOST === 'boolean'; // using client host/port by default
const hmrPort = usingClientHost ? window.location.port : Number(FARM_HMR_PORT);
const hmrHost = usingClientHost ? window.location.hostname : FARM_HMR_HOST;

const socketProtocol =
  FARM_HMR_PROTOCOL || (location.protocol === 'https:' ? 'wss' : 'ws');
const socketHostUrl = `${hmrHost}:${hmrPort}${FARM_HMR_PATH}`;

export class HmrClient {
  socket: WebSocket;
  registeredHotModulesMap = new Map<string, HotModuleState>();
  disposeMap = new Map<string, (data: any) => void | Promise<void>>();
  pruneMap = new Map<string, (data: any) => void | Promise<void>>();
  customListenersMap = new Map<
    string,
    ((data: any) => void | Promise<void>)[]
  >();

  private pageActive = true;
  private pageHidden = false;
  private reloadAllowed = true;
  private recovery: AbortController | undefined;

  constructor(private moduleSystem: ModuleSystem) {
    window.addEventListener('beforeunload', () => {
      const socket = this.socket;
      const wasDisconnected = socket?.readyState === WebSocket.CLOSED;
      this.pageActive = false;
      this.reloadAllowed = false;
      this.recovery?.abort();
      // A canceled navigation leaves its socket open. Firefox starts closing
      // the socket before pagehide, even while the next document is loading.
      setTimeout(() => {
        if (this.pageHidden || this.socket !== socket) return;
        if (socket?.readyState < WebSocket.CLOSING) {
          this.pageActive = true;
          this.reloadAllowed = true;
        } else if (wasDisconnected) {
          // An already disconnected socket cannot reveal whether navigation
          // was canceled. Reconnect without reloading the departing document.
          this.pageActive = true;
          this.connect();
        }
      }, 0);
    });
    window.addEventListener('pagehide', () => {
      // Firefox closes WebSockets when navigating. This is not a server restart.
      this.pageActive = false;
      this.pageHidden = true;
      this.recovery?.abort();
      if (this.socket) this.closeConnectionGracefully();
    });
    window.addEventListener('pageshow', (event) => {
      if (event.persisted && !this.pageActive) {
        this.pageHidden = false;
        this.pageActive = true;
        this.reloadAllowed = true;
        this.connect();
      }
    });
    const resumeReload = (event: Event) => {
      // Reconnecting alone cannot distinguish cancellation from a slow
      // navigation. Input in this document confirms that it remains in use.
      if (event.isTrusted && this.pageActive && !this.pageHidden) {
        const wasSuppressed = !this.reloadAllowed;
        this.reloadAllowed = true;
        if (wasSuppressed && this.socket?.readyState === WebSocket.CLOSED) {
          this.connect();
        }
      }
    };
    window.addEventListener('pointerdown', resumeReload);
    window.addEventListener('keydown', resumeReload);
  }

  connect() {
    this.recovery?.abort();
    logger.debug('connecting to the server...');

    // setup websocket connection
    const socket = new WebSocket(
      `${socketProtocol}://${socketHostUrl}`,
      'farm_hmr'
    );

    this.socket = socket;
    // listen for the message from the server
    // when the user save the file, the server will recompile the file(and its dependencies as long as its dependencies are changed)
    // after the file is recompiled, the server will generated a update resource and send its id to the client
    // the client will apply the update

    socket.addEventListener('message', (event) => {
      const result: HMRPayload = new Function(`return (${event.data})`)();
      if (result?.type === 'closing') {
        this.closeConnectionGracefully();
        return;
      }
      this.handleMessage(result);
    });

    socket.addEventListener(
      'open',
      () => {
        this.notifyListeners('vite:ws:connect', { webSocket: socket });
        this.notifyListeners('farm:ws:connect', { webSocket: socket });
      },
      { once: true }
    );

    socket.addEventListener('close', async () => {
      if (!this.pageActive || this.socket !== socket) return;
      // TODO Do you want to do an elegant cleaning?
      // if (wasClean) return;

      this.notifyListeners('vite:ws:disconnect', { webSocket: socket });
      this.notifyListeners('farm:ws:disconnect', { webSocket: socket });

      if (!this.reloadAllowed) return;

      logger.debug(
        'disconnected from the server, Please refresh the page manually. If you still encounter errors, this may be a farm bug. Please submit an issue. https://github.com/farm-fe/farm/issues'
      );

      this.recovery?.abort();
      const recovery = new AbortController();
      this.recovery = recovery;
      const serverAvailable = await waitForSuccessfulPing(
        socketProtocol,
        socketHostUrl,
        recovery.signal
      );
      if (
        serverAvailable &&
        !recovery.signal.aborted &&
        this.pageActive &&
        this.reloadAllowed &&
        this.socket === socket
      ) {
        location.reload();
      }
    });

    return socket;
  }

  closeConnectionGracefully() {
    if (
      this.socket.readyState === WebSocket.CLOSING ||
      this.socket.readyState === WebSocket.CLOSED
    ) {
      return;
    }
    this.socket.close(1000, 'Client closing connection');
  }

  async applyHotUpdates(result: HmrUpdateResult, moduleSystem: ModuleSystem) {
    result.changed.forEach((id) => {
      logger.debug(`${id} updated`);
    });

    for (const id of result.removed) {
      const prune = this.pruneMap.get(id);
      if (prune) {
        const hotContext = this.registeredHotModulesMap.get(id);
        await prune(hotContext.data);
      }

      // if the module is both in remove and added, we just need to clear the cache
      if (!result.added.includes(id)) {
        moduleSystem.e(id);
      }

      this.registeredHotModulesMap.delete(id);
    }

    for (const id of result.added) {
      // module is already registered, just clear the cache
      moduleSystem.a(id);
    }

    for (const id of result.changed) {
      if (!result.boundaries[id]) {
        // do not found boundary module, reload the window
        location.reload();
      }
    }

    if (result.dynamicResources && result) {
      moduleSystem.sd(
        result.dynamicResources,
        result.dynamicModuleResourcesMap
      );
    }

    for (const chains of Object.values(result.boundaries)) {
      for (const chain of chains) {
        // clear the cache of the boundary module and its dependencies
        for (const id of chain) {
          moduleSystem.a(id);
        }

        try {
          // require the boundary module
          const boundary = chain[chain.length - 1];
          const hotContext = this.registeredHotModulesMap.get(boundary);
          const acceptedDep =
            chain.length > 1 ? chain[chain.length - 2] : undefined;

          if (!hotContext) {
            logger.debug(
              `hot context is empty for boundary ${boundary}. Hot update of ${boundary} is skipped.`
            );
            // location.reload();
            // fix multi page application hmr
            continue;
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

              const acceptedExports = moduleSystem.r(acceptedId);

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
          logger.error(`Error occurred while applying hot updates: ${err}`);
          location.reload();
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
      case 'error': {
        this.notifyListeners('vite:error', payload);
        this.notifyListeners('farm:error', payload);
        if (payload.overlay) createOverlay(payload.err);
        break;
      }
      case 'connected':
        logger.debug('connected to the server');
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
        location.reload();
        break;
      case 'prune':
        this.notifyListeners('vite:beforePrune', payload);
        this.notifyListeners('farm:beforePrune', payload);
        break;

      default:
        logger.warn(`unknown message payload: ${payload}`);
    }
  }

  handleFarmUpdate(result: RawHmrUpdateResult) {
    hasErrorOverlay() && clearOverlay();

    // #fix 2262, using eval instead of new Function to avoid the context issue in micro frontend
    eval(`${result.immutableModules}`);
    eval(`${result.mutableModules}`);

    this.applyHotUpdates(
      {
        added: result.added,
        changed: result.changed,
        removed: result.removed,
        boundaries: result.boundaries,
        // modules,
        dynamicResources: result.dynamicResources,
        dynamicModuleResourcesMap: result.dynamicModuleResourcesMap
      },
      this.moduleSystem
    );
  }
}

export function createOverlay(err: any) {
  clearOverlay();
  document.body.appendChild(new ErrorOverlay(err));
}

function clearOverlay() {
  document.querySelectorAll<ErrorOverlay>(overlayId).forEach((n) => n.close());
}

function hasErrorOverlay() {
  return document.querySelectorAll(overlayId).length;
}

export function waitForWindowShow(signal?: AbortSignal) {
  return new Promise<void>((resolve) => {
    const finish = () => {
      document.removeEventListener('visibilitychange', onChange);
      signal?.removeEventListener('abort', finish);
      resolve();
    };
    const onChange = () => {
      if (document.visibilityState === 'visible') {
        finish();
      }
    };
    if (signal?.aborted) return resolve();
    document.addEventListener('visibilitychange', onChange);
    signal?.addEventListener('abort', finish, { once: true });
  });
}

async function waitForSuccessfulPing(
  socketProtocol: string,
  hostAndPath: string,
  signal: AbortSignal,
  ms = 1000
) {
  const pingHostProtocol = socketProtocol === 'wss' ? 'https' : 'http';

  const ping = async () => {
    // A fetch on a websocket URL will return a successful promise with status 400,
    // but will reject a networking error.
    // When running on middleware mode, it returns status 426, and an cors error happens if mode is not no-cors
    try {
      await fetch(`${pingHostProtocol}://${hostAndPath}`, {
        signal,
        mode: 'no-cors',
        headers: {
          // Custom headers won't be included in a request with no-cors so (ab)use one of the
          // safelisted headers to identify the ping request
          Accept: 'text/x-farm-ping'
        }
      });
      return !signal.aborted;
    } catch {
      /* empty */
    }
    return false;
  };

  if (await ping()) {
    return true;
  }
  await wait(ms, signal);

  while (!signal.aborted) {
    if (document.visibilityState === 'visible') {
      if (await ping()) {
        return true;
      }
      await wait(ms, signal);
    } else {
      await waitForWindowShow(signal);
    }
  }
  return false;
}

export function wait(ms: number, signal?: AbortSignal) {
  return new Promise<void>((resolve) => {
    if (signal?.aborted) return resolve();
    const finish = () => {
      clearTimeout(timer);
      signal?.removeEventListener('abort', finish);
      resolve();
    };
    const timer = setTimeout(finish, ms);
    signal?.addEventListener('abort', finish, { once: true });
  });
}
