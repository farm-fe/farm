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
        farmRequire.o(exports, "Comp1", function() {
            return Comp1;
        });
        farmRequire.o(exports, "Comp2", function() {
            return Comp2;
        });
        function Comp1() {
            console.log('Comp1');
        }
        function Comp2() {
            console.log('Comp2');
        }
        var _f_exports3 = farmRequire("69233311");
        farmRequire._e(exports, _f_exports3);
    },
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "unstable_batchUpdate", function() {
            return unstable_batchUpdate;
        });
        var _f_exports = farmRequire("52961596");
        farmRequire._e(exports, _f_exports);
        var _f_exports1 = farmRequire("027594c8");
        farmRequire._e(exports, _f_exports1);
        function unstable_batchUpdate() {
            console.log('unstable_batchUpdate');
        }
    },
    "52961596": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Provider", function() {
            return Provider;
        });
        farmRequire.o(exports, "useDispatch", function() {
            return useDispatch;
        });
        farmRequire.o(exports, "useStore", function() {
            return useStore;
        });
        function Provider() {
            return 'Provider';
        }
        function useDispatch() {
            return 'useDispatch';
        }
        function useStore() {
            return 'useStore';
        }
    },
    "69233311": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Comp3", function() {
            return Comp3;
        });
        function Comp3() {
            console.log('Comp3');
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.Provider, _f_dep.unstable_batchUpdate, _f_dep.Comp1, _f_dep.Comp2, _f_dep.Comp3);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");