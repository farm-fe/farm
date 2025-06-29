//farm_internal_runtime_index.js:
 const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);


//index-2528a0d0.css:
 .farm-base {
  font-size: 20px;
}
.farm-hide {
  display: none;
}
.farm-show {
  display: block;
}
 .farm-hello {
  color: blue;
}

//index.js:
 import "./farm_internal_runtime_index.js";import "./index-2528a0d0.css";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "95fe6ac5": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        "";
        exports.default = {
            "base": `farm-base`,
            "hello": `farm-hello`,
            "hide": `farm-hide`,
            "show": `farm-show`
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_index = farmRequire.i(farmRequire("95fe6ac5"));
        console.log(farmRequire.f(_f_index).base);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");