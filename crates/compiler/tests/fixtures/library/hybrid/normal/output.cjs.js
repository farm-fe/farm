//index.js:
 var __farmNodeRequire = require;
defineExportEsModule(exports);
exportByDefineProperty(exports, "bar", ()=>bar);
exportByDefineProperty(exports, "default", ()=>index_ts_default);
exportByDefineProperty(exports, "foo", ()=>foo);
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
; // module_id: @farmfe/runtime/src/modules/module-helper.ts
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
function importDefault(v) {
    if (typeof v.default !== 'undefined') {
        return v.default;
    }
    // compatible with `import default from "module"`
    return v;
}
; // module_id: index.ts
var farmRequire$1 = farmRegister("index.ts", function(module, exports1) {
    defineExportEsModule(exports1);
    exportByDefineProperty(exports1, "foo", ()=>foo);
    exportByDefineProperty(exports1, "bar", ()=>bar);
    var _f_node_fs = interopRequireDefault(__farmNodeRequire('node:fs'));
    const os = __farmNodeRequire('node:os');
    console.log(importDefault(_f_node_fs).read, os.cpus);
    exports1.default = {
        read: importDefault(_f_node_fs).read,
        c: 1
    };
    var foo = 'foo';
    var bar = 'bar';
});
var __farm_cjs_exports__$1 = farmRequire$1();
var foo = __farm_cjs_exports__$1.foo, index_ts_default = interopRequireDefault(__farm_cjs_exports__$1).default, bar = __farm_cjs_exports__$1.bar;
