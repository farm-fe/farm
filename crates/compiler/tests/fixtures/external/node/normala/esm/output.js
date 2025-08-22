//index.js:
 import { createRequire } from 'module';var require = createRequire(import.meta.url);(function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());import * as __farm_external_module_jquery from "jquery";
global['__farm_default_namespace__'].m.se({
    "jquery": __farm_external_module_jquery
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = import.meta.url;
        moduleSystem.g(moduleId, module);
    }
})(global["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_jquery = farmRequire.i(farmRequire('jquery'));
        console.log(farmRequire.f(_f_jquery).find);
    }
});
var __farm_ms__ = global['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");