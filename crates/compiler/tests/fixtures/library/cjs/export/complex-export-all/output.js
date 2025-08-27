//index.js:
 ; // module_id: @farmfe/runtime/src/module-system.ts
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
; // module_id: dep.cjs
var farmRequire = farmRegister("dep.cjs", function(module, exports) {
    module.exports.Chart = function Chart() {
        return "CJS Chart Component";
    };
});
var __farm_cjs_exports__$1 = farmRequire();
var Chart = __farm_cjs_exports__$1.Chart;
; // module_id: core.js
; // module_id: chart.js
function Chart$1() {
    return Chart();
}
; // module_id: reexport.ts
; // module_id: index.ts
console.log(Chart$1);
