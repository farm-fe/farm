//index.js:
 import { readFile as default$1 } from "node:fs";
import { foo } from "/external/foo";
import { unstable_batchedUpdates as unstable_batchedUpdates$2 } from "/external/react-dom";
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
    module.exports.unstable_batchedUpdates = function unstable_batchedUpdates() {
        console.log("unstable_batchedUpdates");
    };
});
var __farm_cjs_exports__$1 = farmRequire();
var unstable_batchedUpdates = __farm_cjs_exports__$1.unstable_batchedUpdates;
; // module_id: reexport.ts
; // module_id: index.ts
const unstable_batchedUpdates$1 = 123;
console.log({
    unstable_batchedUpdates: unstable_batchedUpdates$1
});
console.log({
    r1: default$1,
    foo: foo,
    batch: unstable_batchedUpdates$2,
    unstable_batchedUpdates1: unstable_batchedUpdates
});
