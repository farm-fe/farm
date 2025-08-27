//dep-cab3b8f2.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "dep_cab3b8f2d71e03ba887a7e612f30ad14_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "3e3af5b6": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "foo", function() {
            return foo;
        });
        function foo() {
            return 1;
        }
    }
});


//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('dynamic-import.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
initModuleSystem$1(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_dcdc3e0b3362edb8fec2a51d3fa51f8f_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmDynamicRequire("3e3af5b6").then(({ foo })=>{
            console.log(foo());
        });
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.si([]);__farm_ms__.sd([{ path: 'dep-cab3b8f2.js', type: 0 }],{ '3e3af5b6': [0] });__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;