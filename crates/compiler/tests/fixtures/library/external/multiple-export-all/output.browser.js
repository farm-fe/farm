//index.js:
 (function(){if(typeof window==='undefined'||!window['__farm_default_namespace__']||!window['__farm_default_namespace__'].m){; // module_id: @farmfe/runtime/src/module-system.ts.farm-runtime
function setGlobalRequire(globalObj) {
    // polyfill require when running in browser or node with Farm runtime
    const __global_this__ = typeof globalObj !== 'undefined' ? globalObj : {};
    __global_this__.require = __global_this__.require || farmRequire$1;
}
{
    const __global_this__ = typeof globalThis !== 'undefined' ? globalThis : self;
    // keep runtime-generated references like `window[...]` / `global[...]` working in workers and other runtime environments by poly-filling a global `window` variable if it doesn't exist
    if (typeof __global_this__.window === 'undefined') {
        __global_this__.window = __global_this__;
    }
    setGlobalRequire(__global_this__);
}// all modules registered
const __farm_internal_modules__ = {};
// module cache after module initialized
const __farm_internal_cache__ = {};
var __farm_internal_module_system__ = {
    r: farmRequire$1,
    g: farmRegister,
    m: ()=>__farm_internal_modules__,
    c: ()=>__farm_internal_cache__
};
{
    __farm_internal_module_system__.te = // @ts-ignore injected during compile time
    // eslint-disable-next-line no-undef
    "browser";
}{
    // externalModules
    __farm_internal_module_system__.em = {};
    // The external modules are injected during compile time.
    __farm_internal_module_system__.se = function setExternalModules(externalModules) {
        for(const key in externalModules){
            let em = externalModules[key];
            // add a __esModule flag to the module if the module has default export
            if (em && em.default && !em.__esModule) {
                em = {
                    ...em,
                    __esModule: true
                };
            }
            __farm_internal_module_system__.em[key] = em;
        }
    };
    // init global namespace container
    const __farm_global_this__ = (window['__farm_default_namespace__'] = {});
    __farm_global_this__.m = __farm_internal_module_system__;
}function farmRequire$1(id) {
    if (__farm_internal_cache__[id]) {
        var cachedModuleResult = __farm_internal_cache__[id].initializer || __farm_internal_cache__[id].exports;
        return cachedModuleResult;
    }
    const initializer = __farm_internal_modules__[id];
    if (!initializer) {
        {
            // externalModules
            if (__farm_internal_module_system__.em[id]) {
                return __farm_internal_module_system__.em[id];
            }
        }
        console.debug(`[Farm] Module "${id}" is not registered`);
        // return a empty module if the module is not registered
        return {};
    }
    // create a full new module instance and store it in cache to avoid cyclic initializing
    const module = __farm_internal_cache__[id] = {
        id,
        meta: {
            env: {}
        },
        exports: {},
        require: farmRequire$1
    };
    __farm_internal_cache__[id] = module;
    {
        const result = initializer(module, module.exports, farmRequire$1, __farm_internal_module_system__.d);
        // it's a async module, return the promise
        if (result && result instanceof Promise) {
            module.initializer = result.then(()=>{
                module.initializer = undefined;
                // return the exports of the module
                return module.exports;
            });
            return module.initializer;
        }
    }
    // return the exports of the module
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$1(id);
}
; // module_id: @farmfe/runtime/src/modules/dynamic-import.ts.farm-runtime
let dynamicResources = [];
// dynamic module entry and resources map
let dynamicModuleResourcesMap = {};
const loadedResources = {};
const loadingResources = {};
// available public paths, when loading resources, we will try each publicPath until it is available, this is so called `resource loading retry`
const publicPaths = [];
const pendingImports = new Set();
let resourceTransaction;
let resourceRevision = 0;
let moduleSystem;
function initModuleSystem(ms) {
    moduleSystem = ms;
    moduleSystem.pp = publicPaths;
    moduleSystem.d = (id)=>trackImport(dynamicImport(id));
    moduleSystem.sp = setPublicPaths;
    moduleSystem.si = setInitialLoadedResources;
    moduleSystem.sd = setDynamicModuleResourcesMap;
    moduleSystem.gd = getDynamicModuleResourcesMap;
    moduleSystem.l = (id, force)=>trackImport(loadDynamicResourcesOnly(id, force));
    moduleSystem.beginResourceTransaction = beginResourceTransaction;
}
function trackImport(promise) {
    resourceRevision++;
    const owner = resourceTransaction;
    pendingImports.add(promise);
    void promise.then(()=>{
        pendingImports.delete(promise);
    }, (error)=>{
        pendingImports.delete(promise);
        if (owner && !owner.errors.includes(error)) owner.errors.push(error);
    });
    return promise;
}
async function drainImports() {
    while(pendingImports.size)await Promise.allSettled([
        ...pendingImports
    ]);
}
async function beginResourceTransaction() {
    if (resourceTransaction) throw new Error('Dynamic resource transaction already active');
    do {
        await drainImports();
    }while (pendingImports.size);
    if (resourceTransaction) throw new Error('Dynamic resource transaction already active');
    const mapping = getDynamicModuleResourcesMap();
    const previousLoaded = {
        ...loadedResources
    };
    const previousPaths = [
        ...publicPaths
    ];
    const transaction = {
        errors: [],
        elements: new Set()
    };
    resourceTransaction = transaction;
    let closing = false;
    let validatedRevision;
    let settledRevision;
    const assertOpen = ()=>{
        if (closing || resourceTransaction !== transaction) throw new Error('Dynamic resource transaction closed');
        closing = true;
    };
    const isSettled = ()=>!closing && resourceTransaction === transaction && settledRevision === resourceRevision && !pendingImports.size;
    const settle = async ()=>{
        assertOpen();
        try {
            do {
                await drainImports();
            }while (pendingImports.size);
            settledRevision = resourceRevision;
        } finally{
            closing = false;
        }
    };
    const validate = async ()=>{
        validatedRevision = undefined;
        do {
            await settle();
        }while (!isSettled());
        if (transaction.errors.length) throw new AggregateError(transaction.errors, 'Dynamic resource transaction failed');
        validatedRevision = settledRevision;
    };
    const restore = ()=>{
        if (closing || resourceTransaction !== transaction) throw new Error('Dynamic resource transaction closed');
        if (!isSettled()) throw new Error('Dynamic resource transaction must be settled before restoration');
        for (const element of transaction.elements)element.remove();
        for (const key of Object.keys(loadedResources))delete loadedResources[key];
        Object.assign(loadedResources, previousLoaded);
        setDynamicModuleResourcesMap(mapping.resources, mapping.moduleResources);
        publicPaths.splice(0, publicPaths.length, ...previousPaths);
        resourceTransaction = undefined;
        return [
            ...transaction.errors
        ];
    };
    const finalize = ()=>{
        if (closing || resourceTransaction !== transaction) throw new Error('Dynamic resource transaction closed');
        if (validatedRevision !== resourceRevision || !isSettled() || transaction.errors.length) {
            throw new Error('Dynamic resource transaction must be validated after its latest work');
        }
        resourceTransaction = undefined;
    };
    return {
        settle,
        isSettled,
        restore,
        validate,
        finalize,
        async commit () {
            await validate();
            finalize();
        },
        async rollback () {
            do {
                await settle();
            }while (!isSettled());
            return restore();
        }
    };
}
function requireDynamicModule(id) {
    const exports = moduleSystem.r(id);
    // if the module is async, return the default export, the default export should be a promise
    return exports.__farm_async ? exports.default : Promise.resolve(exports);
}
function dynamicImport(id) {
    if (moduleSystem.m()[id] && !dynamicModuleResourcesMap[id]) {
        return requireDynamicModule(id);
    }
    return loadDynamicResources(id);
}
function loadDynamicResources(id, force = false) {
    const resources = dynamicModuleResourcesMap[id].map((index)=>dynamicResources[index]);
    return loadDynamicResourcesOnly(id, force).then(()=>{
        // Do not require the module if all the resources are not js resources
        if (resources.every((resource)=>resource.type !== 0)) {
            return;
        }
        if (!moduleSystem.m()[id]) {
            throw new Error(`Dynamic imported module "${id}" is not registered.`);
        }
        return requireDynamicModule(id);
    }).catch((err)=>{
        console.error(`[Farm] Error loading dynamic module "${id}"`, err);
        throw err;
    });
}
function loadDynamicResourcesOnly(id, force = false) {
    const resources = dynamicModuleResourcesMap[id].map((index)=>dynamicResources[index]);
    if (!moduleSystem.m()[id] && (!resources || resources.length === 0)) {
        throw new Error(`Dynamic imported module "${id}" does not belong to any resource`);
    }
    // force reload resources
    if (force) {
        moduleSystem.a(id);
    }
    // loading all required resources, and return the exports of the entry module
    return Promise.all(resources.map((resource)=>{
        if (force) {
            const resourceLoaded = isResourceLoaded(resource.path);
            setLoadedResource(resource.path, false);
            if (resourceLoaded) {
                return load(resource, `?t=${Date.now()}`);
            }
        }
        return load(resource);
    }));
}
function load(resource, query) {
    // Promise.all for a page can reject while another script or stylesheet is still loading.
    return trackImport(loadUntracked(resource, query));
}
function loadUntracked(resource, query) {
    {
        if (loadedResources[resource.path] && !query) {
            // Skip inject Promise polyfill for runtime
            return Promise.resolve();
        } else if (loadingResources[resource.path]) {
            if (query) {
                loadingResources[resource.path] = loadingResources[resource.path].then(()=>loadResource(resource, 0, query));
            }
            return loadingResources[resource.path];
        }
        return loadResource(resource, 0, query);
    }
}
function loadResource(resource, index, query) {
    const publicPath = publicPaths[index];
    const url = `${publicPath.endsWith('/') ? publicPath.slice(0, -1) : publicPath}/${resource.path}${query || ''}`;
    let promise = Promise.resolve();
    if (resource.type === 0) {
        promise = loadScript(url);
    } else if (resource.type === 1) {
        promise = loadLink(url);
    }
    const loading = promise.then(()=>{
        loadedResources[resource.path] = true;
        loadingResources[resource.path] = null;
    }).catch((e)=>{
        console.warn(`[Farm] Failed to load resource "${url}" using publicPath: ${publicPaths[index]}`);
        index++;
        if (index < publicPaths.length) {
            return loadResource(resource, index);
        } else {
            loadingResources[resource.path] = null;
            throw new Error(`[Farm] Failed to load resource: "${resource.path}, type: ${resource.type}". ${e}`);
        }
    });
    // Callers must await the retry/error chain itself. Returning the raw DOM
    // promise both bypasses publicPath retries and leaves this rejection unowned.
    loadingResources[resource.path] = loading;
    return loading;
}
function loadScript(path) {
    return new Promise((resolve, reject)=>{
        const script = document.createElement('script');
        resourceTransaction?.elements.add(script);
        script.src = path;
        document.body.appendChild(script);
        script.onload = ()=>{
            resolve();
        };
        script.onerror = (e)=>{
            // A failed element is not a loaded resource and must not survive retry.
            script.remove();
            reject(e);
        };
    });
}
function loadLink(path) {
    return new Promise((resolve, reject)=>{
        const link = document.createElement('link');
        resourceTransaction?.elements.add(link);
        link.rel = 'stylesheet';
        link.href = path;
        document.head.appendChild(link);
        link.onload = ()=>{
            resolve();
        };
        link.onerror = (e)=>{
            // Chromium can expose an empty sheet even for a failed request. Leaving
            // it behind makes consumers retain that stale sheet when deduplicating.
            link.remove();
            reject(e);
        };
    });
}
function setLoadedResource(path, loaded = true) {
    loadedResources[path] = loaded;
}
function isResourceLoaded(path) {
    return loadedResources[path];
}
// The public paths are injected during compile time
function setPublicPaths(p) {
    resourceRevision++;
    for(const key in p){
        publicPaths[key] = p[key];
    }
}
function setInitialLoadedResources(resources) {
    resourceRevision++;
    resources.forEach((resource)=>{
        setLoadedResource(resource);
    });
}
// These two methods are used to support dynamic module loading, the dynamic module info is collected by the compiler and injected during compile time
// This method can also be called during runtime to add new dynamic modules
function setDynamicModuleResourcesMap(dr, dmp) {
    resourceRevision++;
    dynamicResources = dr.map((resource)=>({
            ...resource
        }));
    dynamicModuleResourcesMap = Object.fromEntries(Object.entries(dmp).map(([id, indices])=>[
            id,
            [
                ...indices
            ]
        ]));
}
function getDynamicModuleResourcesMap() {
    return {
        resources: dynamicResources.map((resource)=>({
                ...resource
            })),
        moduleResources: Object.fromEntries(Object.entries(dynamicModuleResourcesMap).map(([id, indices])=>[
                id,
                [
                    ...indices
                ]
            ]))
    };
}
; // module_id: @farmfe/runtime/src/modules/module-system-helper.ts.farm-runtime
let moduleSystem$1;
function initModuleSystem$1(ms) {
    moduleSystem$1 = ms;
    moduleSystem$1.u = updateModule;
    moduleSystem$1.e = deleteModule;
    moduleSystem$1.a = clearCache;
}
function updateModule(moduleId, init) {
    const modules = moduleSystem$1.m();
    modules[moduleId] = init;
    clearCache(moduleId);
}
function deleteModule(moduleId) {
    const modules = moduleSystem$1.m();
    if (modules[moduleId]) {
        clearCache(moduleId);
        delete modules[moduleId];
        return true;
    } else {
        return false;
    }
}
function clearCache(moduleId) {
    const cache = moduleSystem$1.c();
    if (cache[moduleId]) {
        delete cache[moduleId];
        return true;
    } else {
        return false;
    }
}
; // module_id: @farmfe/runtime/src/modules/module-helper.ts.farm-runtime
function initModuleSystem$2(ms) {
    const farmRequire = ms.r;
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
        ms.c()[id] = {
            id,
            exports: {
                defineExportStar
            }
        };
    }
}
function exportByDefineProperty(to, to_k, get) {
    if (Object.prototype.hasOwnProperty.call(to, to_k)) {
        return;
    }
    Object.defineProperty(to, to_k, {
        enumerable: true,
        get
    });
}
function defineExport(to, to_k, val) {
    exportByDefineProperty(to, to_k, function() {
        return val;
    });
}
function defineExportEsModule(to) {
    const key = '__esModule';
    if (to[key]) return;
    Object.defineProperty(to, key, {
        value: true
    });
}
function defineExportStar(to, from) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                value: from[k],
                enumerable: true,
                configurable: true
            });
        }
    });
    return from;
}
function interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
function getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    // @ts-ignore ignore type check
    return (getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}
function interopRequireWildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
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
function defineExportFrom(to, to_k, from, from_k) {
    defineExport(to, to_k, from[from_k || to_k]);
}
function importDefault(v) {
    if (typeof v.default !== 'undefined') {
        return v.default;
    }
    // compatible with `import default from "module"`
    return v;
}
; // module_id: @farmfe/runtime
initModuleSystem(__farm_internal_module_system__);
initModuleSystem$1(__farm_internal_module_system__);
initModuleSystem$2(__farm_internal_module_system__);
}}());window['__farm_default_namespace__'].m.se({
    "/external/bar": window['/external/bar'] || {},
    "/external/zoo": window['/external/zoo'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_e094168e7fa415b98009295e04f081de_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "index.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_zoo = farmRequire.w(farmRequire("/external/zoo"));
        var zoo_ambiguous_export_all_farm_internal_ = _f_zoo;
        var _f_bar = farmRequire.w(farmRequire("/external/bar"));
        var bar_ambiguous_export_all_farm_internal_ = _f_bar;
        var zoo_test = zoo_ambiguous_export_all_farm_internal_.test || bar_ambiguous_export_all_farm_internal_.test;
        ; // module_id: foo.ts
        const foo = 'foo';
        ; // module_id: index.ts
        console.log(zoo_test, foo);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;var __farm_entry__=__farm_ms__.r("index.ts");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;