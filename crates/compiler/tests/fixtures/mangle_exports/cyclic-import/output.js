//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_570ed6fce2e117e84667277ace804e01_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "44a34200": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return hello;
        });
        farmRequire.o(exports, "c", function() {
            return world;
        });
        farmRequire.o(exports, "b", function() {
            return sayHello;
        });
        var _f_zoo = farmRequire("774fba3e");
        const hello = "hello";
        const world = "world";
        function sayHello() {
            console.log(hello, world);
            _f_zoo.a();
        }
    },
    "774fba3e": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return sayZoo;
        });
        farmRequire.o(exports, "b", function() {
            return zoo;
        });
        var _f_export = farmRequire("44a34200");
        const zoo = "zoo";
        function sayZoo() {
            console.log(zoo);
            _f_export.b();
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_export = farmRequire("44a34200");
        console.log(_f_export.b);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;