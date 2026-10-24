//index.js:
 import { createRequire } from 'module';var require = createRequire(import.meta.url);(function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());import { createRequire as __farm_create_require__ } from 'node:module';
global['__farm_default_namespace__'].m.r.nr = __farm_create_require__(import.meta.url);
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = import.meta.url;
        moduleSystem.g(moduleId, module);
    }
})(global["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        const xxx = farmRequire.nr('jquery');
        console.log({
            xxx
        });
    }
});
var __farm_ms__ = global['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;