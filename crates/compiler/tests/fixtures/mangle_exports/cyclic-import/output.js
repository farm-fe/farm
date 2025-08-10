//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "44a34200": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "hello", function() {
            return hello;
        });
        farmRequire.o(exports, "world", function() {
            return world;
        });
        farmRequire.o(exports, "sayHello", function() {
            return sayHello;
        });
        var _f_zoo = farmRequire("774fba3e");
        const hello = "hello";
        const world = "world";
        function sayHello() {
            console.log(hello, world);
            _f_zoo.sayZoo();
        }
    },
    "774fba3e": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "sayZoo", function() {
            return sayZoo;
        });
        farmRequire.o(exports, "zoo", function() {
            return zoo;
        });
        var _f_export = farmRequire("44a34200");
        const zoo = "zoo";
        function sayZoo() {
            console.log(zoo);
            _f_export.sayHello();
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_export = farmRequire("44a34200");
        console.log(_f_export.sayHello);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");