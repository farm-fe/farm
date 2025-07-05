//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(global["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log(module.meta.url);
    }
});
var __farm_ms__ = global['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");