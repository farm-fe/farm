//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-system-helper.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
initModuleSystem$1(__farm_internal_module_system__);
}());window['__farm_default_namespace__'].m.se({
    "node:fs": window['node:fs'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "index.ts": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_node_fs = farmRequire.i(farmRequire('node:fs'));
        console.log(farmRequire.f(_f_node_fs));
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("index.ts");