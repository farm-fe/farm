//farm_internal_runtime_index.js:
 const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);


//index-c9a6.css:
 .farm-base {
  font-size: 18px;
}
.farm-action {
  color: red;
}

//index.js:
 import "./farm_internal_runtime_index.js";import "./index-c9a6.css";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "8b6840d6": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        "";
        exports.default = {
            "action": `farm-action`
        };
    },
    "95fe6ac5": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        "";
        var _f_action = farmRequire.i(farmRequire("8b6840d6"));
        exports.default = {
            "base": `farm-base ${farmRequire.f(_f_action)["action"]}`
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_index = farmRequire.i(farmRequire("95fe6ac5"));
        console.log(farmRequire.f(_f_index).base);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");