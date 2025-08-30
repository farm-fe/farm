//index.js:
 (function(){; // module_id: @farmfe/runtime/src/module-system.ts.farm-runtime
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
; // module_id: @farmfe/runtime/src/modules/module-system-helper.ts.farm-runtime
let moduleSystem;
function initModuleSystem(ms) {
    moduleSystem = ms;
    moduleSystem.u = updateModule;
    moduleSystem.e = deleteModule;
    moduleSystem.a = clearCache;
}
function updateModule(moduleId, init) {
    const modules = moduleSystem.m();
    modules[moduleId] = init;
    clearCache(moduleId);
}
function deleteModule(moduleId) {
    const modules = moduleSystem.m();
    if (modules[moduleId]) {
        clearCache(moduleId);
        delete modules[moduleId];
        return true;
    } else {
        return false;
    }
}
function clearCache(moduleId) {
    const cache = moduleSystem.c();
    if (cache[moduleId]) {
        delete cache[moduleId];
        return true;
    } else {
        return false;
    }
}
; // module_id: @farmfe/runtime/src/modules/module-helper.ts.farm-runtime
function initModuleSystem$1(ms) {
    const farmRequire = ms.r;
    {
        farmRequire.o = exportByDefineProperty;
        // exports.xx = xx
        farmRequire.d = defineExport;
        // exports.__esModule
        farmRequire._m = defineExportEsModule;
    }
    {
        // `export * from` helper
        farmRequire._e = defineExportStar;
        // inject defineExportStar to module system
        const id = '@farm-runtime/module-helper';
        ms.c()[id] = {
            id,
            exports: {
                defineExportStar
            }
        };
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
}());window['__farm_default_namespace__'].m.se({
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
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("index.ts");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;