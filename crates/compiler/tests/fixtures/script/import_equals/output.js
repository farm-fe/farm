//farm_internal_runtime_index.js:
 const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);


//index-7eccb2bd.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "e4b1dea3": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log('fs-extra');
    }
});


//index.js:
 import "./farm_internal_runtime_index.js";import "./index-7eccb2bd.js";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "363fc137": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log('utils.js');
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        const fs = farmRequire("e4b1dea3");
        const utils = farmRequire("363fc137");
        console.log(fs, utils);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");