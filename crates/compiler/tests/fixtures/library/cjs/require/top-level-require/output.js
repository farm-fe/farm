//index.js:
 var __farmNodeRequire = require;
function exportByDefineProperty(to, to_k, get) {
    if (Object.prototype.hasOwnProperty.call(to, to_k)) {
        return;
    }
    Object.defineProperty(to, to_k, {
        enumerable: true,
        get
    });
}
defineExportEsModule(exports);
exportByDefineProperty(exports, "default", ()=>index_ts_default);
; // module_id: @farmfe/runtime/src/module-system.ts
// all modules registered
const __farm_internal_modules__ = {};
// module cache after module initialized
const __farm_internal_cache__ = {};
function farmRequire$2(id) {
    if (__farm_internal_cache__[id]) {
        var cachedModuleResult = __farm_internal_cache__[id].exports;
        return cachedModuleResult;
    }
    const initializer = __farm_internal_modules__[id];
    if (!initializer) {
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
        require: farmRequire$2
    };
    __farm_internal_cache__[id] = module;
    initializer(module, module.exports);
    // return the exports of the module
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$2(id);
}
; // module_id: @farmfe/runtime/src/modules/module-helper.ts
function defineExportEsModule(to) {
    const key = '__esModule';
    if (to[key]) return;
    Object.defineProperty(to, key, {
        value: true
    });
}
; // module_id: index.ts
var farmRequire$1 = farmRegister("index.ts", function(module, exports1) {
    defineExportEsModule(exports1);
    var _f_node_module = __farmNodeRequire('node:module');
    const _f_cjs_require = _f_node_module.createRequire(require("node:url").pathToFileURL(__filename).href);
    console.log(_f_cjs_require('node:fs'));
});
var index_ts_default = farmRequire$1();
