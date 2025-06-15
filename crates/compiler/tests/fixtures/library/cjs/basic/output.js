//index.js:
 var esm_mjs_default = "esm.mjs";
var esm_mjs_namespace_farm_internal_ = {
    default: esm_mjs_default,
    __esModule: true
};
var esm_mjs_default$1 = "cur_esm.mjs";
var esm_mjs_namespace_farm_internal_$1 = {
    default: esm_mjs_default$1,
    __esModule: true
};
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
function interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
var farmRequire$3 = farmRegister("zoo.cjs", function(module, exports) {
    console.log("zoo should be executed after foo");
    module.exports.zoo = `zoo.cjs`;
});
var farmRequire = farmRegister("foo/index.cjs", function(module, exports) {
    const esm = esm_mjs_namespace_farm_internal_;
    const curEsm = esm_mjs_namespace_farm_internal_$1;
    console.log("foo should be executed before zoo");
    const zoo = farmRequire$3();
    module.exports = `foo + ${esm.default} + ${curEsm.default} + ${zoo.zoo}`;
});
var __farm_cjs_exports__$2 = farmRequire();
var index_cjs_default = interopRequireDefault(__farm_cjs_exports__$2).default;
var farmRequire$4 = farmRegister("bar/bar.cjs", function(module, exports) {
    module.exports = {
        bar: "bar.cjs"
    };
});
var farmRequire$1 = farmRegister("bar/index.cjs", function(module, exports) {
    const esm = esm_mjs_namespace_farm_internal_;
    const bar = farmRequire$4();
    module.exports = `bar + ${esm.default} + ${bar.bar}`;
});
var __farm_cjs_exports__$3 = farmRequire$1();
var index_cjs_default$1 = interopRequireDefault(__farm_cjs_exports__$3).default;
var length = index_cjs_default$1.length;
console.log(index_cjs_default, index_cjs_default$1, length);
