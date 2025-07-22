//index.js:
 import { createRequire as __farmNodeCreateRequire } from "module";
var __farmNodeRequire = __farmNodeCreateRequire(import.meta.url);
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
; // module_id: dep.cjs
var farmRequire = farmRegister("dep.cjs", function(module, exports) {
    const fs = __farmNodeRequire("node:fs");
    const path = __farmNodeRequire("node:path");
    module.exports = {
        fs,
        path
    };
});
var __farm_cjs_exports__$2 = farmRequire();
var fs = __farm_cjs_exports__$2.fs, path = __farm_cjs_exports__$2.path;
; // module_id: dep1.cjs
var farmRequire$1 = farmRegister("dep1.cjs", function(module, exports) {
    const fs$1 = __farmNodeRequire("node:fs");
    const path$1 = __farmNodeRequire("node:path");
    module.exports = {
        fs: fs$1,
        path: path$1
    };
});
var __farm_cjs_exports__$3 = farmRequire$1();
var fs$1 = __farm_cjs_exports__$3.fs, path$1 = __farm_cjs_exports__$3.path;
; // module_id: index.ts
console.log('dep.cjs', fs, path);
console.log('dep1.cjs', fs$1, path$1);
