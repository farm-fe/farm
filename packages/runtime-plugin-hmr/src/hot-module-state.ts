import { HmrUpdateResult } from './types';
import type {
  ModuleSystem,
  ModuleInitialization,
} from '@farmfe/runtime/src/module-system';
import { handleErrorSync } from './utils';

const REGISTERED_HOT_MODULES = new Map<string, HotModuleState>();

export class HotModuleState {
  acceptCallbacks: Array<() => void> = [];

  id: string;

  constructor(id: string) {
    this.id = id;
  }

  accept(callback?: () => void) {
    if (callback) {
      this.acceptCallbacks.push(callback);
    }
  }

  tap = (changeModule: ModuleInitialization) => {
    this.acceptCallbacks.map((cb) => {
      handleErrorSync(cb, [changeModule], (err) => {
        console.error(err);
      });
    });
  };
}

export function createHotContext(id: string) {
  if (REGISTERED_HOT_MODULES.has(id)) {
    return REGISTERED_HOT_MODULES.get(id);
  }

  const state = new HotModuleState(id);
  REGISTERED_HOT_MODULES.set(id, state);
  return state;
}

export function applyHotUpdates(
  result: HmrUpdateResult,
  moduleSystem: ModuleSystem
) {
  console.log('applyHotUpdates', result);

  for (const id of result.removed) {
    moduleSystem.delete(id);
    REGISTERED_HOT_MODULES.delete(id);
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

  // TODO support accept dependencies change
  for (const updated_id of Object.keys(result.boundaries)) {
    const chains = result.boundaries[updated_id];

    for (const chain of chains) {
      for (const id of chain) {
        moduleSystem.clearCache(id);
      }

      // require the boundary module
      const boundary = chain[chain.length - 1];
      const boundaryExports = moduleSystem.require(boundary);
      const hotContext = REGISTERED_HOT_MODULES.get(boundary);
      hotContext.tap(boundaryExports);
    }
  }
}
