//farm_internal_runtime_index.js:
 const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);


//index.js:
 import "./farm_internal_runtime_index.js";import "./index_337c.css";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "6f462555": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        "";
        exports.default = 'comp';
    },
    "b3d9bc98": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log('resolved.ts');
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire("b3d9bc98");
        var _f_comp = farmRequire.i(farmRequire("6f462555"));
        console.log(farmRequire.f(_f_comp));
        exports.default = 2;
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_default__=__farm_entry__.default;export {__farm_entry_default__ as default};

//index_337c.css:
 .body {
  color: red;
}