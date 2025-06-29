//index.js:
 defineExportEsModule(exports);
exportByDefineProperty(exports, "bar", ()=>bar);
exportByDefineProperty(exports, "default", ()=>index_ts_default);
exportByDefineProperty(exports, "foo", ()=>foo);
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
var farmRequire = farmRegister("index.ts", function(module, exports) {
    defineExportEsModule(exports);
    exportByDefineProperty(exports, "foo", ()=>foo);
    exportByDefineProperty(exports, "bar", ()=>bar);
    var _f_node_fs = interopRequireDefault(require('node:fs'));
    const os = require('node:os');
    console.log(importDefault(_f_node_fs).read, os.cpus);
    exports.default = {
        read: importDefault(_f_node_fs).read,
        c: 1
    };
    var foo = 'foo';
    var bar = 'bar';
});
var __farm_cjs_exports__$1 = farmRequire();
var index_ts_default = interopRequireDefault(__farm_cjs_exports__$1).default, foo = __farm_cjs_exports__$1.foo, bar = __farm_cjs_exports__$1.bar;
