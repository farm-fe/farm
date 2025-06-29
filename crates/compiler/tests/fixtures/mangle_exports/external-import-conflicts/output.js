//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());window['__farm_default_namespace__'].m.se({
    "/external/deep/unresolved": window['/external/deep/unresolved'] || {},
    "/external/unresolved": window['/external/unresolved'] || {},
    "node:fs": window['node:fs'] || {},
    "node:module": window['node:module'] || {}
});
(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "6d686e48": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_node_fs = farmRequire("node:fs");
        console.log('foo existsSync', _f_node_fs.existsSync('foo'));
    },
    "774fba3e": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_node_module = farmRequire('node:module');
        farmRequire._e(exports, _f_node_module);
        var _f_unresolved = farmRequire('/external/unresolved');
        farmRequire._(exports, "a", _f_unresolved, "unresolved");
        var _f_unresolved1 = farmRequire('/external/deep/unresolved');
        farmRequire._(exports, "b", _f_unresolved1, "unresolvedDeep");
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_bar = farmRequire("c5bba42f");
        farmRequire("6d686e48");
        var _f_bar1 = farmRequire.w(farmRequire("c5bba42f"));
        var ns = _f_bar1;
        var _f_zoo = farmRequire("774fba3e");
        var _f_zoo1 = farmRequire("774fba3e");
        var _f_unresolved = farmRequire('/external/deep/unresolved');
        console.log('index readFileSync', _f_bar.a('index'));
        console.log(_f_zoo.a, _f_zoo1.b, _f_unresolved.b);
        console.log(ns);
        console.log(_f_bar.createRequire);
    },
    "c5bba42f": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_node_fs = farmRequire('node:fs');
        console.log('bar existsSync', _f_node_fs.existsSync('bar'));
        var _f_node_fs1 = farmRequire('node:fs');
        farmRequire._(exports, "a", _f_node_fs1, "existsSync");
        var _f_zoo = farmRequire("774fba3e");
        farmRequire._e(exports, _f_zoo);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");