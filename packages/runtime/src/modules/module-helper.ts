import type { Module, ModuleSystem } from "../module-system.js";

// Opaque prebuilt Farm bundles share the host module system. Their helper needs
// cannot be inferred from the host's imports, so this is a stable runtime ABI.
export function initModuleSystem(ms: ModuleSystem) {
  const farmRequire: any = ms.r;
  farmRequire.entryBindings ??= createEntryLiveBindings(ms);
  farmRequire.o = exportByDefineProperty;
  farmRequire.d = defineExport;
  farmRequire._m = defineExportEsModule;
  farmRequire._e = defineExportStar;
  farmRequire.i = interopRequireDefault;
  farmRequire._g = getRequireWildcardCache;
  farmRequire.w = interopRequireWildcard;
  farmRequire._ = defineExportFrom;
  farmRequire.f = importDefault;

  const id = '@farm-runtime/module-helper';
  // Preserve existing cached exports when initializing helpers on a shared host.
  if (!ms.c()[id]) {
    ms.c()[id] = { id, exports: { defineExportStar } };
  }
}

export function exportByDefineProperty(to: any, to_k: string, get: () => any) {
  if (Object.prototype.hasOwnProperty.call(to, to_k)) {
    return;
  }
  Object.defineProperty(to, to_k, {
    enumerable: true,
    get
  });
}

export function defineExport(to: any, to_k: string, val: any) {
  exportByDefineProperty(to, to_k, function () {
    return val;
  });
}

// exports.__esModule
export function defineExportEsModule(to: any) {
  const key = '__esModule';
  if (to[key]) return;
  Object.defineProperty(to, key, { value: true });
}

// `export * from` helper
export function defineExportStar(to: any, from: any) {
  Object.keys(from).forEach(function (k) {
    if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
      Object.defineProperty(to, k, {
        get: function () { return from[k]; },
        enumerable: true,
        configurable: true
      });
    }
  });

  return from;
}

// `import xxx from` helper
export function interopRequireDefault(obj: any) {
  return obj && obj.__esModule ? obj : { default: obj };
}

function getRequireWildcardCache(nodeInterop: any) {
  if (typeof WeakMap !== "function") return null;

  var cacheBabelInterop = new WeakMap();
  var cacheNodeInterop = new WeakMap();

  // @ts-ignore ignore type check
  return (getRequireWildcardCache = function (nodeInterop: any) {
    return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
  })(nodeInterop);
}

// `import * as xx` helper, copied from @swc/helper
export function interopRequireWildcard(obj: any, nodeInterop: any) {
  if (!nodeInterop && obj && obj.__esModule) return obj;
  if (obj === null || typeof obj !== "object" && typeof obj !== "function") return { default: obj };

  var cache = getRequireWildcardCache(nodeInterop);

  if (cache && cache.has(obj)) return cache.get(obj);

  var newObj: any = { __proto__: null };
  var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;

  for (var key in obj) {
      if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
          var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
          if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
          else newObj[key] = obj[key];
      }
  }

  newObj.default = obj;

  if (cache) cache.set(obj, newObj);

  return newObj;
}

// `export { xx } from` helper
export function defineExportFrom(to: any, to_k: string, from: any, from_k?: string) {
  exportByDefineProperty(to, to_k, function () {
    return from[from_k == null ? to_k : from_k];
  });
}

// minify x.default
export function importDefault(v: any) {
  if(typeof v.default !== 'undefined') {
    return v.default;
  }

  // compatible with `import default from "module"`
  return v;
}

type Listener = (value: unknown) => void;
type Origin = { ready: boolean; owner: Module; read: () => unknown; write?: (value: unknown) => void };
type Subscription = { listener: Listener; initialized: boolean; value: unknown };

/** Internal synchronous bridge for compiled entry exports. It does not scan getters. */
export function createEntryLiveBindings(modules: ModuleSystem) {
  const origins = new Map<string, Origin>();
  const listeners = new Map<string, Map<symbol, Subscription>>();
  const current = (origin: Origin) => modules.c()[origin.owner.id] === origin.owner;
  const deliver = (subscription: Subscription, value: unknown) => {
    subscription.listener(value);
    subscription.value = value;
    subscription.initialized = true;
  };
  const emit = (key: string, value: unknown) => {
    for (const subscription of listeners.get(key)?.values() ?? []) deliver(subscription, value);
  };
  return {
    bind(key: string, owner: Module, read: () => unknown, write: (value: unknown) => void) {
      const origin = { ready: false, owner, read, write };
      if (current(origin)) origins.set(key, origin);
      // Registration happens only after a successful write. An early TDZ or
      // stale async factory must not replace a currently initialized origin.
      return {
        get value() { return read(); },
        set value(value: unknown) {
          write(value);
          if (current(origin)) {
            origin.ready = true;
            origins.set(key, origin);
            emit(key, read());
          }
        }
      };
    },
    seed(key: string, owner: Module, read: () => unknown) {
      const origin: Origin = { ready: true, owner, read };
      if (!current(origin)) return;
      const value = read();
      origins.set(key, origin);
      emit(key, value);
    },
    complete(owner: Module, keys: string[]) {
      for (const key of keys) {
        const origin = origins.get(key);
        if (origin && origin.owner === owner && current(origin)) {
          const value = origin.read();
          origin.ready = true;
          emit(key, value);
        }
      }
    },
    subscribe(key: string, listener: Listener, ...seed: [unknown?]) {
      let bucket = listeners.get(key);
      if (!bucket) listeners.set(key, bucket = new Map());
      const token = Symbol();
      const subscription = { listener, initialized: false, value: undefined as unknown };
      bucket.set(token, subscription);
      if (seed.length) deliver(subscription, seed[0]);
      else {
        const origin = origins.get(key);
        if (origin?.ready && current(origin)) deliver(subscription, origin.read());
      }
      return () => {
        const active = listeners.get(key);
        active?.delete(token);
        if (active?.size === 0) listeners.delete(key);
      };
    },
    snapshot() {
      const saved = new Map([...origins].filter(([, origin]) => origin.ready).map(([key, origin]) => [key, { origin, value: origin.read() }]));
      const savedListeners = new Map([...listeners].map(([key, bucket]) => [key, new Map([...bucket].map(([token, subscription]) => [token, { ...subscription }]))]));
      return () => {
        origins.clear();
        listeners.clear();
        for (const [key, bucket] of savedListeners) listeners.set(key, new Map([...bucket].map(([token, subscription]) => [token, { ...subscription }])));
        for (const [key, { origin, value }] of saved) {
          origins.set(key, origin);
          origin.write?.(value);
        }
        // Native slots have their own transaction state. A cleared source can
        // legitimately differ from the last value delivered to a native slot.
        for (const bucket of listeners.values()) for (const subscription of bucket.values()) {
          if (subscription.initialized) subscription.listener(subscription.value);
        }
      };
    }
  };
}

export type EntryLiveBindings = ReturnType<typeof createEntryLiveBindings>;
