//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};const a = 3;
const b = 4;
const c = 5;
function BB() {
    const a$1 = 5;
    const b$1 = 6;
    console.log(a$1, b$1);
}
var dep_ts_default = {
    a: a,
    b: b,
    c: c
};
var dep_ts_ns = {
    a: a,
    b: b,
    "default": dep_ts_default,
    __esModule: true
};



var exportAll_ts_ns = {
    a: a,
    b: b,
    __esModule: true
};

console.log({
    ExportNamespace: dep_ts_ns,
    A: exportAll_ts_ns,
    ImportNamespace: dep_ts_ns
});
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");