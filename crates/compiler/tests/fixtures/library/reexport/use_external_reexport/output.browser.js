//index.js:
 (function(){; // module_id: 9cd7578f
function setGlobalRequire(globalThis) {
    // polyfill require when running in browser or node with Farm runtime
    const __global_this__ = typeof globalThis !== 'undefined' ? globalThis : {};
    __global_this__.require = __global_this__.require || farmRequire$1;
}
{
    setGlobalRequire(window);
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
    // @ts-ignore injected during compile time
    __farm_internal_module_system__.te = "browser";
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
    // init `window['xxxx] = {}`
    const __farm_global_this__ = window['__farm_default_namespace__'] = {};
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
    initializer(module, module.exports, farmRequire$1, __farm_internal_module_system__.d);
    // return the exports of the module
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$1(id);
}
; // module_id: ae1b52c0
function initModuleSystem(ms) {
    const farmRequire = ms.r;
    {
        farmRequire.o = exportByDefineProperty;
        // exports.xx = xx
        farmRequire.d = defineExport;
        // exports.__esModule
        farmRequire._m = defineExportEsModule;
    }
    {
        // `import xxx from` helper
        farmRequire.i = interopRequireDefault;
    }
    {
        // `import * as xx` helper, copied from @swc/helper
        farmRequire._g = getRequireWildcardCache;
        // `import * as xx` helper, copied from @swc/helper
        farmRequire.w = interopRequireWildcard;
    }
    {
        farmRequire._ = defineExportFrom;
    }
    {
        farmRequire.f = importDefault;
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
; // module_id: 20764041
initModuleSystem(__farm_internal_module_system__);
}());window['__farm_default_namespace__'].m.se({
    "/external/foo": window['/external/foo'] || {},
    "/external/react-dom": window['/external/react-dom'] || {},
    "node:fs": window['node:fs'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_8628134d6efc61be8d56e055c7028a8b_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "01800dfe": function(module, exports, farmRequire, farmDynamicRequire) {
        module.exports.unstable_batchedUpdates = function unstable_batchedUpdates() {
            console.log("unstable_batchedUpdates");
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_node_fs = farmRequire("node:fs");
        var _f_foo = farmRequire("/external/foo");
        var _f_react_dom = farmRequire("/external/react-dom");
        var _f_dep = farmRequire("01800dfe");
        ; // module_id: aa67029b
        ; // module_id: b5d64806
        const unstable_batchedUpdates = 123;
        console.log({
            unstable_batchedUpdates
        });
        console.log({
            r1: _f_node_fs.readFile,
            foo: _f_foo.foo,
            batch: _f_react_dom.unstable_batchedUpdates,
            unstable_batchedUpdates1: _f_dep.unstable_batchedUpdates
        });
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;