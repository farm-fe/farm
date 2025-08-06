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
    "027594c8": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Comp1;
        });
        farmRequire.o(exports, "b", function() {
            return Comp2;
        });
        function Comp1() {
            console.log('Comp1');
        }
        function Comp2() {
            console.log('Comp2');
        }
        var _f_exports3 = farmRequire("69233311");
        farmRequire._(exports, "c", _f_exports3, "a");
    },
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return unstable_batchUpdate;
        });
        var _f_exports = farmRequire("52961596");
        farmRequire._(exports, "b", _f_exports, "a");
        var _f_exports1 = farmRequire("027594c8");
        farmRequire._(exports, "c", _f_exports1, "b");
        farmRequire._(exports, "d", _f_exports1, "c");
        farmRequire._(exports, "e", _f_exports1, "a");
        function unstable_batchUpdate() {
            console.log('unstable_batchUpdate');
        }
    },
    "52961596": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Provider;
        });
        function Provider() {
            return 'Provider';
        }
    },
    "69233311": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "a", function() {
            return Comp3;
        });
        function Comp3() {
            console.log('Comp3');
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.b, _f_dep.a, _f_dep.e, _f_dep.c, _f_dep.d);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");