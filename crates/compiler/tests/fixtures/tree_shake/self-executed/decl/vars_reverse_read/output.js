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
        farmRequire.o(exports, "a1", function() {
            return a1;
        });
        farmRequire.o(exports, "a2", function() {
            return a2;
        });
        farmRequire.o(exports, "a3", function() {
            return a3;
        });
        const a1 = {};
        const a2 = {};
        const b2 = {
            a2
        };
        b2.a2.aaa = 2;
        const a3 = {};
        const b3 = {
            a3
        };
        console.log(b3);
        const c3 = {
            b3
        };
        console.log(c3);
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.a1, _f_dep.a2, _f_dep.a3);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");