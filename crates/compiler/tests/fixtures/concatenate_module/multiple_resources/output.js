//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('dynamic-import.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
initModuleSystem$1(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmDynamicRequire("817bf312").then((mod)=>console.log(mod));
        farmDynamicRequire("b5906cd8").then((mod)=>console.log(mod));
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.si([]);__farm_ms__.sd([{ path: 'route1-251db514.js', type: 0 },{ path: 'route2-7e737731.js', type: 0 },{ path: 'route2-b6a18655.js', type: 0 }],{ '817bf312': [0,1],'b5906cd8': [1,2] });__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");

//route1-251db514.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "817bf312": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Route1", function() {
            return Route1;
        });
        var _f_common1 = farmRequire("dfcad1dc");
        function Route1Comp() {
            return 'Route1Comp';
        }
        function Route1() {
            return Route1Comp() + _f_common1.Common1();
        }
    }
});


//route2-7e737731.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "dfcad1dc": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Common1", function() {
            return Common1;
        });
        farmRequire.o(exports, "Common2", function() {
            return Common2;
        });
        function Common2() {
            return 'Common2';
        }
        function Common1() {
            return 'Common1' + Common2();
        }
    }
});


//route2-b6a18655.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5906cd8": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Route2", function() {
            return Route2;
        });
        var _f_common1 = farmRequire("dfcad1dc");
        var _f_common11 = farmRequire.w(farmRequire("dfcad1dc"));
        var common1_ts_external_all_farm_internal_ = _f_common11;
        var Common2 = common1_ts_external_all_farm_internal_.Common2;
        function Common3() {
            return 'Common3';
        }
        function Route2() {
            return "Route2" + _f_common1.Common1() + Common2() + Common3();
        }
    }
});
