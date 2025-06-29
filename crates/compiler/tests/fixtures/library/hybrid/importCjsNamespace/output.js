//index.js:
 ; // module_id: @farm-runtime/module-system
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
        require: (moduleId)=>farmRequire$1(moduleId)
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
; // module_id: @farm-runtime/module-helper
function interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
; // module_id: lodash.ts
var farmRequire = farmRegister("lodash.ts", function(module, exports) {
    module.exports.name = 'lodash';
    module.exports.default = 'foo';
});
var __farm_cjs_exports__$1 = farmRequire();
var lodash_ts_default = interopRequireDefault(__farm_cjs_exports__$1).default;
; // module_id: a.ts
const lodash = 'a.ts';
console.log(lodash, 'a.ts');
; // module_id: b.ts
console.log('b.ts', __farm_cjs_exports__$1);
; // module_id: index.ts
console.log('index.ts', lodash_ts_default);
