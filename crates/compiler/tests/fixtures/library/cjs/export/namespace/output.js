//index.js:
 const __farm_internal_modules__ = {};
const __farm_internal_cache__ = {};
function farmRequire$2(id) {
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
        require: (moduleId)=>farmRequire$2(moduleId)
    };
    __farm_internal_cache__[id] = module;
    initializer(module, module.exports);
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$2(id);
}
var farmRequire = farmRegister("dep.cjs", function(module, exports) {
    module.exports.default = "dep.cjs";
    module.exports.__esModule = true;
    module.exports.length = 1;
});
var __farm_cjs_exports__$2 = farmRequire();
var farmRequire$1 = farmRegister("dep1.cjs", function(module, exports) {
    module.exports.foo = "dep1.cjs foo";
    module.exports.bar = "dep1.cjs bar";
});
var __farm_cjs_exports__$3 = farmRequire$1();
var foo = __farm_cjs_exports__$3.foo, bar = __farm_cjs_exports__$3.bar;
console.log(__farm_cjs_exports__$2, foo, bar);
