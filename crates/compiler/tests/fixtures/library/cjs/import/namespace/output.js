//index.js:
 const __farm_internal_modules__ = {};
const __farm_internal_cache__ = {};
function farmRequire$1(id) {
    if (__farm_internal_cache__[id]) {
        var cachedModuleResult = __farm_internal_cache__[id].exports;
        return cachedModuleResult;
    }
    const initializer = __farm_internal_modules__[id];
    if (!initializer) {
        console.debug(`[Farm] Module "${id}" is not registered`);
        return {};
    }
    const module = __farm_internal_cache__[id] = {
        id,
        meta: {
            env: {}
        },
        exports: {},
        require: (moduleId)=>farmRequire$1(moduleId)
    };
    __farm_internal_cache__[id] = module;
    initializer(module, module.exports);
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$1(id);
}
var farmRequire = farmRegister("dep.cjs", function(module, exports) {
    module.exports.dep = "dep";
});
var __farm_cjs_exports__$1 = farmRequire();
console.log(__farm_cjs_exports__$1);
