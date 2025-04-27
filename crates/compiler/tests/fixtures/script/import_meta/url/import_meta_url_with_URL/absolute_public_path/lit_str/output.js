//foo-276e63.txt?url:
 foo

//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
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
        exports.default = "/foo-276e63.txt?url";
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");