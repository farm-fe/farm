//debounce-6f74.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "01609b59": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "default", function() {
            return debounce;
        });
        var _f_utils = farmRequire("a5831d05");
        function debounce(fn) {
            _f_utils.debug("debounce");
            return fn;
        }
    }
});


//farm_internal_runtime_index.js:
 const moduleSystem = {};
function initModuleSystem() {
    console.log('dynamic-import.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
initModuleSystem$1(moduleSystem);


//index-2faa.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "a5831d05": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "debug", function() {
            return debug;
        });
        function debug(msg) {
            console.log(msg);
        }
    }
});


//index-64d2.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "726cd210": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "default", function() {
            return merge;
        });
        var _f_utils = farmRequire("a5831d05");
        function merge(a, b) {
            _f_utils.debug("merge");
            return a + b;
        }
    }
});


//index.js:
 import "./farm_internal_runtime_index.js";import "./index-2faa.js";import "./index-64d2.js";window['__farm_default_namespace__'].m.se({
    "module": window['module'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_module = farmRequire.i(farmRequire('module'));
        var _f_merge = farmRequire.i(farmRequire("726cd210"));
        function defineConfig(config) {
            farmRequire.f(_f_merge)(config, {
                compilation: {
                    input: {
                        main: './main.tsx'
                    },
                    external: farmRequire.f(_f_module).builtinModules
                }
            });
            return config;
        }
        farmDynamicRequire("01609b59").then((debounce)=>{
            console.log(debounce);
        });
        exports.default = defineConfig({});
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.si(['index-2faa.js','index-64d2.js']);__farm_ms__.sd([{ path: 'index-2faa.js', type: 0 },{ path: 'debounce-6f74.js', type: 0 }],{ '01609b59': [0,1] });__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");var __farm_entry_default__=__farm_entry__.default;export {__farm_entry_default__ as default};