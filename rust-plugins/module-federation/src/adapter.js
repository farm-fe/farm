import { createInstance } from __MF_RUNTIME_PATH__;

// A synchronous bridge installation; no asynchronous Farm bootstrap hook.
const configuration = __MF_CONFIGURATION__;
const system = globalThis[configuration.namespace].m;
if (system.federation) throw new Error(`[Farm MF ${configuration.name}] Adapter already installed`);
const shared = __MF_SHARED__;
const preservePendingSingletons = {
  name: 'farm-native-singleton-selection',
  beforeInitContainer(data) {
    for (const [id, config] of Object.entries(shared)) {
      if (!config.shareConfig.singleton) continue;
      const entry = data.shareScope?.[id]?.[config.version];
      if (entry?.from !== configuration.name || entry.get !== config.get ||
          entry.loaded || typeof entry.loading?.then !== 'function') continue;

      // Webpack uses loaded to prevent replacing an already selected provider.
      // Publish the selected async getter without delaying container init.
      const pending = entry.loading;
      const priorLoaded = entry.loaded;
      const priorGet = entry.get;
      const selectedGet = () => pending;
      entry.get = selectedGet;
      entry.loaded = true;
      void pending.then(
        () => {
          if (entry.get === selectedGet) entry.get = priorGet;
        },
        () => {
          if (entry.get !== selectedGet) return;
          entry.get = priorGet;
          if (entry.loading !== pending) return;
          if (!entry.lib) entry.loaded = priorLoaded;
          delete entry.loading;
        },
      );
    }
    return data;
  },
};
const instance = createInstance({name:configuration.name,remotes:configuration.remotes,shared,shareStrategy:'loaded-first',plugins:[preservePendingSingletons]});
__MF_HOT_ADAPTER__
const hot = configuration.hot ? createFederationHot(configuration) : undefined;
export const bridge = {
  acceptRemote: (id, callback) => {if(!hot) return ()=>{};return hot.accept(id,callback);},
  publish: (key, value) => hot?.publish(key,value),
  installHotBoundary: context => hot?.install(context),
  instance,
  async loadShared(id) {
    try {
      const factory = await instance.loadShare(id);
      if (typeof factory !== 'function') throw new Error('Shared factory unavailable');
      return factory();
    } catch(cause) {
      throw new Error(`[Farm MF ${configuration.name}] Shared ${id}: ${cause.message}`,{cause});
    }
  },
  async loadRemote(id) {
    try {
      const value = await instance.loadRemote(id);
      if (value == null) throw new Error('Runtime returned no module');
      return hot ? hot.consume(id,value) : value;
    } catch (cause) {
      const remote = configuration.remotes.find(remote => id.startsWith(remote.alias + '/'));
      throw new Error(`[Farm MF ${configuration.name}] Failed ${id} from ${remote?.entry}: ${cause.message}`,{cause});
    }
  },
  createContainer(loaders) {
    let initialized = false;
    let scope;
    let initializing;
    const token = {from:configuration.name};
    return {
      init(nextScope, initScope = [], options = {}) {
        if (initializing && nextScope !== scope) return Promise.reject(new Error(`[Farm MF ${configuration.name}] Different share scope`));
        if (initScope.includes(token)) return Promise.resolve();
        if (initializing) return initializing;
        const scopeName = options.shareScopeKeys || 'default';
        if (typeof scopeName !== 'string') return Promise.reject(new Error('Only one share scope is supported'));
        scope = nextScope;
        initScope.push(token);
        initializing = Promise.resolve().then(async () => {
          instance.initShareScopeMap(scopeName,nextScope,{hostShareScopeMap:options.shareScopeMap});
          await Promise.all(instance.initializeSharing(scopeName,{initScope,from:'build',strategy:'loaded-first'}));
          initialized = true;
        });
        return initializing;
      },
      async get(key) {
        if (initializing) await initializing;
        if (!initialized) throw new Error(`[Farm MF ${configuration.name}] init must run before get`);
        if (!Object.prototype.hasOwnProperty.call(loaders,key)) throw new Error(`[Farm MF ${configuration.name}] Unknown expose: ${key}`);
        const namespace = await loaders[key]();
        return () => namespace;
      }
    };
  }
};
system.federation = bridge;
