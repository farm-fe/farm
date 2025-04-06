//farm_internal_runtime_index.js:
 const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);


//index-3b2ec523.css:
 * {
  margin: 0;
  padding: 0;
  background: url("/logo-73d4a8.png");
}

//index.js:
 import "./farm_internal_runtime_index.js";import "./index-3b2ec523.css";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "44a34200": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "c", function() {
            return c;
        });
        "";
        var c = 2;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_export = farmRequire("44a34200");
        console.log(_f_export.c);
    },
    "def8d5dc": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = "/logo-73d4a8.png";
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");

//logo-73d4a8.png:
 