import type { ModuleSystem } from '@farmfe/runtime';
import { HmrUpdateResult, RawHmrUpdateResult } from './types';
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
  pruneMap = new Map<string, (data: any[]) => void | Promise<void>>();

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
      const result: RawHmrUpdateResult = eval(`(${event.data})`);
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
    });

    socket.addEventListener('open', () => {
      logger.log('connected to the server');
    });
    // TODO use ping/pong to detect the connection is closed, and if the server is online again, reload the page
    // socket.addEventListener('close', () => setTimeout(connect, 3000));

    return socket;
  }

  removeCssStyles(removed: string[]) {
    for (const id of removed) {
      const previousStyle = document.querySelector(
        `style[data-farm-id="${{ id }}"]`
      );

      if (previousStyle) {
        previousStyle.remove();
      }
    }
  }

  applyHotUpdates(result: HmrUpdateResult, moduleSystem: ModuleSystem) {
    result.changed.forEach((id) => {
      console.log(`[Farm HMR] ${id} updated`);
    });

    for (const id of result.removed) {
      moduleSystem.delete(id);
      this.registeredHotModulesMap.delete(id);
    }

    this.removeCssStyles(result.removed);

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

    // TODO support accept dependencies change
    for (const updated_id of Object.keys(result.boundaries)) {
      const chains = result.boundaries[updated_id];

      for (const chain of chains) {
        for (const id of chain) {
          moduleSystem.clearCache(id);
        }

        try {
          // require the boundary module
          const boundary = chain[chain.length - 1];
          const boundaryExports = moduleSystem.require(boundary);
          const hotContext = this.registeredHotModulesMap.get(boundary);
          hotContext.tap(boundaryExports);
        } catch (err) {
          // The boundary module's dependencies may not present in current module system for a multi-page application. We should reload the window in this case.
          // See https://github.com/farm-fe/farm/issues/383
          console.error(err);
          window.location.reload();
        }
      }
    }
  }
}
