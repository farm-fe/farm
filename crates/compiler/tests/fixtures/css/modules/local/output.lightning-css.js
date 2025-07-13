//farm_internal_runtime_index.js:
 const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);


//index-2528a0d0.css:
 ._8Z4fiW_base {
  font-size: 20px;
}

._8Z4fiW_hide {
  display: none;
}

._8Z4fiW_show {
  display: block;
}

._8Z4fiW_hello {
  color: #00f;
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
            "base": `_8Z4fiW_base`,
            "hello": `_8Z4fiW_hello`,
            "hide": `_8Z4fiW_hide`,
            "show": `_8Z4fiW_show`
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_index = farmRequire.i(farmRequire("95fe6ac5"));
        console.log(farmRequire.f(_f_index).base);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");