//index.js:
 ; // module_id: @farmfe/runtime/src/module-system.ts
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
; // module_id: lib/move.js
var farmRequire$2 = farmRegister("lib/move.js", function(module, exports) {
    module.exports.move = function move() {
        console.log("move");
    };
});
; // module_id: lib/fs.js
var farmRequire$3 = farmRegister("lib/fs.js", function(module, exports) {
    module.exports = {
        readFileSync: (path, encoding)=>{
            return "console.log('readFileSync')";
        }
    };
});
; // module_id: lib/index.js
var farmRequire = farmRegister("lib/index.js", function(module, exports) {
    module.exports = {
        ...farmRequire$2(),
        ...farmRequire$3()
    };
});
var __farm_cjs_exports__$1 = farmRequire();
var move = __farm_cjs_exports__$1.move, readFileSync = __farm_cjs_exports__$1.readFileSync;
; // module_id: index.ts
console.log(move());
console.log(readFileSync());
