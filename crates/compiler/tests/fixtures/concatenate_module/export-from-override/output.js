//entry1-024632c1.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "e91e7771": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "abstract", function() {
            return abstract;
        });
        var _f_g_util = farmRequire("642f0a6f");
        function isAllowCapture(element) {
            return element.cfg.visible && element.cfg.capture;
        }
        function abstract() {
            console.log(_f_g_util.isString('abs'), _f_g_util.isNil(null), isAllowCapture(123));
        }
    }
});


//entry1-bb290f84.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "933f95a4": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_g_base = farmRequire("e91e7771");
        console.log(_f_g_base.abstract);
    }
});


//entry2-642d8ef7.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "2704a53c": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_g_util = farmRequire("642f0a6f");
        console.log(_f_g_util.isString("entry2"));
    }
});


//entry2-c46e2630.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "642f0a6f": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "isNil", function() {
            return isNil;
        });
        farmRequire.o(exports, "isString", function() {
            return isString;
        });
        function isString(value) {
            return typeof value === "string";
        }
        function isNil(value) {
            return value === null;
        }
    }
});


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
        farmDynamicRequire("933f95a4");
        farmDynamicRequire("2704a53c");
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.si([]);__farm_ms__.sd([{ path: 'entry2-c46e2630.js', type: 0 },{ path: 'entry1-024632c1.js', type: 0 },{ path: 'entry1-bb290f84.js', type: 0 },{ path: 'entry2-642d8ef7.js', type: 0 }],{ '933f95a4': [0,1,2],'2704a53c': [0,3] });__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");