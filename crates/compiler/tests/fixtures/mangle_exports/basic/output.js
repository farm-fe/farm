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
    "44a34200": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "b", function() {
            return world;
        });
        farmRequire.o(exports, "default", function() {
            return sayHello;
        });
        farmRequire.o(exports, "a", function() {
            return hello;
        });
        const hello = 'hello';
        const world = 'world';
        function sayHello() {
            console.log(hello, world);
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_export = farmRequire.w(farmRequire("44a34200"));
        farmRequire.f(_f_export)();
        console.log(_f_export.a, _f_export.b);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");