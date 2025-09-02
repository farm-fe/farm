//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
function foo() {}
var dep_ts_ns = {
    "default": foo,
    __esModule: true
};

var export_ts_cjs = __commonJs((module, exports)=>{
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    _export_star(dep_ts_ns, exports);
    console.log(foo, dep_ts_ns);
    function __commonJs$1() {}
    function _mergeNamespaces$1() {}
    function _getRequireWildcardCache$1() {}
    function _interop_require_wildcard$1() {}
    function _export_star$1() {}
    function _interop_require_default$1() {}
    const dep_ts_ns$1 = {};
    module.exports.name = "shulan";
    module.exports.age = 18;
});

export_ts_cjs();
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){var filename = ((function(){return import.meta.url})());for(var r in _){_[r].__farm_resource_pot__=filename;global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");