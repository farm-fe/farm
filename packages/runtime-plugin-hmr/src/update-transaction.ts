import type { ModuleSystem } from '@farmfe/runtime';
import type { HotModuleState } from './hot-module-state.js';

export interface UpdateTransactionParticipant {
  /** All fallible asynchronous preparation for commit belongs here. */
  validate?(): void | Promise<void>;
  /** Recheck after all asynchronous validators, before any snapshot is released. */
  ready?(): boolean;
  /** Release snapshots only. Throwing during finalization is unrecoverable. */
  commit?(): void;
  /** Drain external work and restore participant state before module restoration. */
  rollback?(): void | Promise<void>;
  /** Re-render/reconnect after module and hot registrations have been restored. */
  recover?(): void | Promise<void>;
}

export type PrepareUpdateTransaction = (modules: ModuleSystem) => UpdateTransactionParticipant | Promise<UpdateTransactionParticipant>;

function recordSnapshot(object: object) {
  const descriptors = Object.getOwnPropertyDescriptors(object);
  return () => {
    for (const key of Reflect.ownKeys(object)) {
      if (!Object.prototype.hasOwnProperty.call(descriptors, key)) Reflect.deleteProperty(object, key);
    }
    Object.defineProperties(object, descriptors);
  };
}

function mapSnapshot<K, V>(map: Map<K, V>, copy: (value: V) => V = value => value) {
  const entries = [...map].map(([key, value]) => [key, copy(value)] as const);
  return () => { map.clear(); for (const [key, value] of entries) map.set(key, value); };
}

/** Restores registry structure; arbitrary nested exports/data or disposer side effects need participants. */
export function captureUpdateSnapshot(
  modules: ModuleSystem,
  hotModules: Map<string, HotModuleState>,
  maps: Map<any, any>[],
  listeners: Map<string, ((data: any) => void | Promise<void>)[]>
) {
  const restoreFactories = recordSnapshot(modules.m());
  const restoreCache = recordSnapshot(modules.c());
  const restoreHotMap = mapSnapshot(hotModules);
  const hotStates = [...hotModules.values()].map(hot => {
    const callbacks = hot.acceptCallbacks.map(callback => ({ ...callback, deps: [...callback.deps] }));
    const data = hot.data;
    const restoreData = recordSnapshot(data);
    return () => { hot.acceptCallbacks = callbacks; hot.data = data; restoreData(); };
  });
  const restoreMaps = maps.map(map => mapSnapshot(map));
  const restoreListeners = mapSnapshot(listeners, callbacks => [...callbacks]);
  return () => {
    const errors: unknown[] = [];
    for (const restore of [restoreFactories, restoreCache, restoreHotMap, ...hotStates, ...restoreMaps, restoreListeners]) {
      try { restore(); } catch (error) { errors.push(error); }
    }
    if (errors.length) throw new AggregateError(errors, 'Hot update registry restoration failed');
  };
}
