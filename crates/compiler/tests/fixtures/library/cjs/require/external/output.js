//index.js:
 import { createRequire as __farmNodeCreateRequire } from "module";
var __farmNodeRequire = __farmNodeCreateRequire(import.meta.url);
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
; // module_id: dep.cjs
var farmRequire = farmRegister("dep.cjs", function(module, exports) {
    const { readFileSync } = __farmNodeRequire("fs");
    console.log(readFileSync("./index.ts", "utf8"));
});
farmRequire();
; // module_id: index.ts
const loaders = {
    '.js': __farmNodeRequire,
    '.cjs': __farmNodeRequire,
    '.json': __farmNodeRequire
};
var index_ts_default = 'require-external';
export { index_ts_default as default, loaders as loaders };
