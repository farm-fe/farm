//index.js:
 import { createRequire } from 'module';var require = createRequire(import.meta.url);(function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(global["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log(import.meta.url);
    }
});
var __farm_ms__ = global['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");