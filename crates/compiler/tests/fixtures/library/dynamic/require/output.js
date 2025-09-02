//index.js:
 ; // module_id: @farmfe/runtime/src/module-system.ts.farm-runtime
// all modules registered
const __farm_internal_modules__ = {};
// module cache after module initialized
const __farm_internal_cache__ = {};
function farmRequire$1(id) {
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
        require: farmRequire$1
    };
    __farm_internal_cache__[id] = module;
    initializer(module, module.exports);
    // return the exports of the module
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$1(id);
}
; // module_id: dep.js
var farmRequire$2 = farmRegister("dep.js", function(module, exports) {
    const dep = "dep";
    module.exports = {
        dep
    };
    module.exports.default = module.exports;
});
var __farm_cjs_exports__$2 = farmRequire$2();
; // module_id: loader.js
var farmRequire = farmRegister("loader.js", function(module, exports) {
    exports.loadTsSync = ()=>farmRequire$2();
    exports.loadTs = async ()=>(await Promise.resolve(__farm_cjs_exports__$2)).default;
});
var __farm_cjs_exports__$3 = farmRequire();
var loadTsSync = __farm_cjs_exports__$3.loadTsSync, loadTs = __farm_cjs_exports__$3.loadTs;
; // module_id: index.ts
console.log(loadTsSync());
console.log(await loadTs());
