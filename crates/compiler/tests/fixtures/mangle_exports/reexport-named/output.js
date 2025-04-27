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
    "027594c8": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Alpha;
        });
        function Alpha() {
            return 'Alpha';
        }
        exports.default = Alpha;
        var _f_exports2 = farmRequire("405aeea7");
        farmRequire._e(exports, _f_exports2);
    },
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return MyAlpha;
        });
        var _f_exports1 = farmRequire("027594c8");
        var _f_exports11 = farmRequire("027594c8");
        farmRequire._(exports, "c", _f_exports11, "a");
        farmRequire._(exports, "b", _f_exports11, "a");
        class MyAlpha {
            constructor(){
                return _f_exports1.a;
            }
        }
    },
    "405aeea7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Checkboard;
        });
        function Checkboard() {
            return 'Checkboard';
        }
        exports.default = Checkboard;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.c, _f_dep.b, _f_dep.a);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");