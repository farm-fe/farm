//index.js:
 ; // module_id: @farm-runtime/module-system
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
; // module_id: @farm-runtime/module-helper
function interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
; // module_id: loadash.cjs
var farmRequire$1 = farmRegister("loadash.cjs", function(module, exports) {
    // @ts-nocheck
    function lodash() {}
    lodash.merge = function() {};
    const _ = lodash;
    (module.exports = _)._ = _;
});
var __farm_cjs_exports__$1 = farmRequire$1();
var loadash_cjs_default = interopRequireDefault(__farm_cjs_exports__$1).default, merge = __farm_cjs_exports__$1.merge;
; // module_id: utils.ts
function print(msg) {
    console.log('print', msg);
}
; // module_id: bundle2.ts
function bundle2() {
    print('bundle2');
}
var bundle2_ts_default = 'default bundle2';
var bundle2_ts_namespace_farm_internal_ = {
    bundle2: bundle2,
    default: bundle2_ts_default,
    __esModule: true
};
; // module_id: logger.ts
class Logger {
    log(msg) {
        print(msg);
    }
}
; // module_id: bundle3.ts
function bundle3() {
    const logger = new Logger();
    logger.log('bundle3');
}
var bundle3_ts_default = 'default bundle3';
var bundle3_ts_namespace_farm_internal_ = {
    bundle3: bundle3,
    default: bundle3_ts_default,
    __esModule: true
};
; // module_id: index.ts
console.log(loadash_cjs_default, merge);
Promise.resolve(bundle2_ts_namespace_farm_internal_).then((mod)=>{
    console.log(mod);
});
Promise.resolve(bundle3_ts_namespace_farm_internal_).then((mod)=>{
    console.log(mod);
});
