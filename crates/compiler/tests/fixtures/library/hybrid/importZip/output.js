//index.js:
 import { readFile } from "node:fs";
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
; // module_id: cjs.ts
var farmRequire$1 = farmRegister("cjs.ts", function(module, exports) {
    defineExportEsModule(exports);
    exportByDefineProperty(exports, "default", ()=>cjs);
    module.exports.cjsName = 'foo';
    module.exports.cjsAge = 18;
    function cjs() {}
});
var __farm_cjs_exports__$1 = farmRequire$1();
var cjsAge = __farm_cjs_exports__$1.cjsAge, cjs = interopRequireDefault(__farm_cjs_exports__$1).default, cjsName = __farm_cjs_exports__$1.cjsName;
; // module_id: esm.ts
const esmName = 'esm';
const esmAge = 19;
function esm() {}
; // module_id: bundle2.ts
const bundle2Name = 'bundle2';
const bundle2Age = 18;
function bundle2() {}
; // module_id: bar.ts
console.log({
    cjs: {
        cjs: cjs,
        cjsName: cjsName
    },
    readFile,
    esm: {
        esm: esm,
        esmName: esmName
    },
    bundle2: {
        bundle2: bundle2,
        bundle2Name: bundle2Name
    }
}, 'bar.ts');
; // module_id: foo.ts
console.log({
    cjs: {
        cjs: cjs,
        cjsAge: cjsAge
    },
    esm: {
        esm: esm,
        esmAge: esmAge
    },
    bundle2: {
        bundle2: bundle2,
        bundle2Age: bundle2Age
    },
    readFile: readFile
}, 'foo.ts');
; // module_id: index.ts
