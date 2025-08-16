//index.js:
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
        require: (moduleId)=>farmRequire$2(moduleId)
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
; // module_id: dep.cjs
var farmRequire = farmRegister("dep.cjs", function(module, exports) {
    module.exports.default = "dep.cjs";
    module.exports.__esModule = true;
    module.exports.length = 1;
});
var __farm_cjs_exports__$2 = farmRequire();
; // module_id: dep1.cjs
var farmRequire$1 = farmRegister("dep1.cjs", function(module, exports) {
    module.exports.foo = "dep1.cjs foo";
    module.exports.bar = "dep1.cjs bar";
});
var __farm_cjs_exports__$3 = farmRequire$1();
var foo = __farm_cjs_exports__$3.foo, bar = __farm_cjs_exports__$3.bar;
; // module_id: ns.mjs
; // module_id: index.ts
console.log(__farm_cjs_exports__$2, foo, bar);
