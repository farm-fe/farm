//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());window['__farm_default_namespace__'].m.se({
    "node:fs": window['node:fs'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_node_fs = farmRequire.i(farmRequire("node:fs"));
        function readFileSync(path) {
            return farmRequire.f(_f_node_fs).readFileSync(path, "utf-8");
        }
        function main(e) {
            console.log(readFileSync('./dep.js'), farmRequire.f(_f_node_fs).readFileSync('./dep.js'), e);
        }
        main('hello world');
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");