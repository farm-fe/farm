//index.js:
 defineExportEsModule(exports);
exportByDefineProperty(exports, "bar", ()=>bar);
exportByDefineProperty(exports, "foo", ()=>foo);
; // module_id: @farmfe/runtime/src/module-system.ts.farm-runtime
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
; // module_id: @farmfe/runtime/src/modules/module-helper.ts.farm-runtime
function exportByDefineProperty(to, to_k, get) {
    if (Object.prototype.hasOwnProperty.call(to, to_k)) {
        return;
    }
    Object.defineProperty(to, to_k, {
        enumerable: true,
        get
    });
}
function defineExportEsModule(to) {
    const key = '__esModule';
    if (to[key]) return;
    Object.defineProperty(to, key, {
        value: true
    });
}
function interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
; // module_id: foo.ts
var farmRequire$1 = farmRegister("foo.ts", function(module, exports1) {
    defineExportEsModule(exports1);
    exportByDefineProperty(exports1, "foo", ()=>foo);
    exportByDefineProperty(exports1, "bar", ()=>bar);
    exports1.default = 'foo';
    var foo = 'foo';
    var bar = 'bar';
    module.exports.cjs = true;
});
var __farm_cjs_exports__$1 = farmRequire$1();
var foo = __farm_cjs_exports__$1.foo, foo_ts_default = interopRequireDefault(__farm_cjs_exports__$1).default, bar = __farm_cjs_exports__$1.bar;
; // module_id: index.ts
