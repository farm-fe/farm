//farm_internal_runtime_index.js:
 const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);


//index-337cc548.css:
 @-webkit-keyframes anim {}
@keyframes anim {}
@-webkit-keyframes anim {
  0% {
    color: red;
  }
}
@keyframes anim {
  0% {
    color: red;
  }
}

//index.js:
 import "./farm_internal_runtime_index.js";import "./index-337cc548.css";(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        "";
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");