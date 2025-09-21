//foo-276e63.txt:
 foo

//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_2f831927b6ab0a68699832bdb583360c_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_foo = farmRequire.i(farmRequire("e6cee430"));
        console.log(new URL({
            "./foo.txt": farmRequire.f(_f_foo)
        }["./foo.txt"], module.meta.url));
    },
    "e6cee430": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = "/foo-276e63.txt";
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;