//index.js:
 ; // module_id: esm.mjs
var esm_mjs_default = "esm.mjs";
var esm_mjs_namespace_farm_internal_ = {
    default: esm_mjs_default,
    __esModule: true
};
; // module_id: foo/esm.mjs
var esm_mjs_default$1 = "cur_esm.mjs";
var esm_mjs_namespace_farm_internal_$1 = {
    default: esm_mjs_default$1,
    __esModule: true
};
; // module_id: @farmfe/runtime/src/module-system.ts
// all modules registered
const __farm_internal_modules__ = {};
// module cache after module initialized
const __farm_internal_cache__ = {};
function farmRequire$3(id) {
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
        require: farmRequire$3
    };
    __farm_internal_cache__[id] = module;
    initializer(module, module.exports);
    // return the exports of the module
    return module.exports;
}
function farmRegister(id, module) {
    __farm_internal_modules__[id] = module;
    return ()=>farmRequire$3(id);
}
; // module_id: @farmfe/runtime/src/modules/module-helper.ts
function interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
; // module_id: zoo.cjs
var farmRequire$4 = farmRegister("zoo.cjs", function(module, exports) {
    console.log("zoo should be executed after foo");
    module.exports.zoo = `zoo.cjs`;
});
; // module_id: foo/index.cjs
var farmRequire$1 = farmRegister("foo/index.cjs", function(module, exports) {
    const esm = esm_mjs_namespace_farm_internal_;
    const curEsm = esm_mjs_namespace_farm_internal_$1;
    console.log("foo should be executed before zoo");
    const zoo = farmRequire$4();
    module.exports = `foo + ${esm.default} + ${curEsm.default} + ${zoo.zoo}`;
});
var __farm_cjs_exports__$2 = farmRequire$1();
var index_cjs_default = interopRequireDefault(__farm_cjs_exports__$2).default;
; // module_id: bar/bar.cjs
var farmRequire$5 = farmRegister("bar/bar.cjs", function(module, exports) {
    module.exports = {
        bar: "bar.cjs"
    };
});
; // module_id: bar/index.cjs
var farmRequire$2 = farmRegister("bar/index.cjs", function(module, exports) {
    const esm = esm_mjs_namespace_farm_internal_;
    const bar = farmRequire$5();
    module.exports = `bar + ${esm.default} + ${bar.bar}`;
});
var __farm_cjs_exports__$3 = farmRequire$2();
var index_cjs_default$1 = interopRequireDefault(__farm_cjs_exports__$3).default;
; // module_id: index.ts
var length = index_cjs_default$1.length;
console.log(index_cjs_default, index_cjs_default$1, length);
