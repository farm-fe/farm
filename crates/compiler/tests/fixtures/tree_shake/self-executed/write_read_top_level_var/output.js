//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_ddf1ac1dd74c1a8307895afa712b1c49_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "bar", function() {
            return bar;
        });
        function foo() {
            console.log("foo");
        }
        function bar() {
            console.log("bar");
        }
        foo.prototype.runFoo = function() {
            console.log("runFoo");
        };
        const tempBar = bar;
        tempBar.prototype.runFoo = foo.prototype.runFoo;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        new _f_dep.bar().runFoo();
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");