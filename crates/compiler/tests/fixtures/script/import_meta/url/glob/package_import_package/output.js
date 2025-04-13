//bar_index-ce66cb21.js:
 (function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
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
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "7546eb2a": function(module, exports, farmRequire, farmDynamicRequire) {
        console.log("@foo/core/foo.js");
    }
});


//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('dynamic-import.ts');
}
function initModuleSystem$1() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
initModuleSystem$1(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
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