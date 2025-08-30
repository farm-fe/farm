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
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_dcdc3e0b3362edb8fec2a51d3fa51f8f_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmDynamicRequire("817bf312").then((mod)=>console.log(mod));
        farmDynamicRequire("b5906cd8").then((mod)=>console.log(mod));
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.si([]);__farm_ms__.sd([{ path: 'route1-21ccd1fe.js', type: 0 },{ path: 'route2-7e737731.js', type: 0 },{ path: 'route2-5dde66a0.js', type: 0 }],{ '817bf312': [0,1],'b5906cd8': [1,2] });__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;

//route1-21ccd1fe.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "route1_21ccd1fe1f70fb8d6f43729f2be149fb_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "01800dfe": function(module, exports, farmRequire, farmDynamicRequire) {
        Object.defineProperty(exports, "registerAction", {
            enumerable: true,
            get: function() {
                return (str)=>"interaction_2.registerAction";
            }
        });
        Object.defineProperty(exports, "registerEngine", {
            enumerable: true,
            get: function() {
                return (str)=>"interaction_2.registerEngine";
            }
        });
    },
    "817bf312": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Route1", function() {
            return Route1;
        });
        var _f_module_helper_ts = farmRequire("@farmfe/runtime/src/modules/module-helper.ts.farm-runtime");
        var _f_dep = farmRequire.w(farmRequire("01800dfe"));
        var dep_cjs_ambiguous_export_all_farm_internal_ = _f_dep;
        var _f_dep1 = farmRequire("01800dfe");
        var _f_common1 = farmRequire("dfcad1dc");
        var _f_dep2 = farmRequire("01800dfe");
        farmRequire._e(exports, _f_dep2);
        var dep_cjs_registerAction = dep_cjs_ambiguous_export_all_farm_internal_.registerAction;
        function Route1Comp() {
            return dep_cjs_registerAction('Route1Comp');
        }
        function Route1() {
            return Route1Comp() + _f_common1.Common1() + _f_dep1.registerEngine('route1');
        }
        var route1_ts_namespace_farm_internal_ = {
            get Route1 () {
                return Route1;
            },
            __esModule: true
        };
        _f_module_helper_ts.defineExportStar(route1_ts_namespace_farm_internal_, dep_cjs_ambiguous_export_all_farm_internal_);
    }
});


//route2-5dde66a0.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "route2_5dde66a0d315dc1862394aba0df9544c_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "2dc19cb0": function(module, exports, farmRequire, farmDynamicRequire) {
        exports.isCjs = true;
        exports.dep2 = function(str) {
            return "dep2" + str;
        };
    },
    "b5906cd8": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Route2", function() {
            return Route2;
        });
        var _f_module_helper_ts = farmRequire("@farmfe/runtime/src/modules/module-helper.ts.farm-runtime");
        var _f_common1 = farmRequire("dfcad1dc");
        var _f_dep2 = farmRequire.w(farmRequire("2dc19cb0"));
        var dep2_cjs_ambiguous_export_all_farm_internal_ = _f_dep2;
        var _f_dep21 = farmRequire("2dc19cb0");
        farmRequire._e(exports, _f_dep21);
        function Common3() {
            return 'Common3';
        }
        function Route2Comp() {
            return route2_ts_namespace_farm_internal_.dep2('Route2Comp');
        }
        function Route2() {
            return "Route2" + _f_common1.Common1() + _f_common1.Common2() + Common3() + Route2Comp();
        }
        var route2_ts_namespace_farm_internal_ = {
            get Route2 () {
                return Route2;
            },
            __esModule: true
        };
        _f_module_helper_ts.defineExportStar(route2_ts_namespace_farm_internal_, dep2_cjs_ambiguous_export_all_farm_internal_);
    }
});


//route2-7e737731.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "route2_7e737731f5e042f86c07888b66039f69_js";
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
