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
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "foo1", function() {
            return foo1;
        });
        farmRequire.o(exports, "foo2", function() {
            return foo2;
        });
        farmRequire.o(exports, "foo3", function() {
            return foo3;
        });
        farmRequire.o(exports, "foo4", function() {
            return foo4;
        });
        function foo() {
            console.log('hello world');
        }
        let foo1 = 1, foo2 = 2;
        foo1 = 2;
        var foo3 = foo;
        var foo4 = foo;
        foo3.create = foo;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.foo1, _f_dep.foo2, _f_dep.foo3, _f_dep.foo4);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");