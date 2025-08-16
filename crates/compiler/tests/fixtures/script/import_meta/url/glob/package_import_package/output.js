//bar_index-ce66cb21.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "bar_index_ce66cb2109b8303ed15a66faaf228652_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "3b411c90": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log({
            "../@foo/core/foo.js": ()=>farmDynamicRequire("7546eb2a")
        });
    }
});


//foo-482fd791.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "foo_482fd791bf978d934829bb6c403df55e_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "7546eb2a": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log("@foo/core/foo.js");
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
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_dcdc3e0b3362edb8fec2a51d3fa51f8f_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log({
            "./node_modules/bar/index.js": ()=>farmDynamicRequire("3b411c90")
        });
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.si([]);__farm_ms__.sd([{ path: 'foo-482fd791.js', type: 0 },{ path: 'bar_index-ce66cb21.js', type: 0 }],{ '7546eb2a': [0],'3b411c90': [1] });__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");