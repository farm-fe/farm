//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};const namedA = 10;
const namedB = 20;
const namedC = 30;
const sameNameWithFile_ts = 1;
const sameNameWithFile_ts_ns$1 = 2;
var exportNamed_ts_default = {
    renamedA: namedA,
    renamedB: namedB,
    renamedC: namedC
};
var exportNamed_ts_ns = {
    namedA: namedA,
    namedB: namedB,
    namedC: namedC,
    renamedA: namedA,
    renamedB: namedB,
    renamedC: namedC,
    "default": exportNamed_ts_default,
    __esModule: true
};

console.log('export expr');
var exportExpr_ts_default = 'export expr';

const sameNameWithFile_ts_ns$2 = 1;
const sameNameWithFile_ts$1 = 2;
const exportExpr_ts_default$1 = 3;
function say() {
    console.log('hello');
}
var sameNameWithFile_ts_ns = {
    say: say,
    __esModule: true
};

console.log({
    NamedNamespace: exportNamed_ts_ns,
    namedA: namedA,
    namedB: namedB,
    namedC: namedC,
    DefaultNamed: exportNamed_ts_default,
    SameNameWithFileNamespace: sameNameWithFile_ts_ns,
    Expr: exportExpr_ts_default
});
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){var filename = ((function(){return import.meta.url})());for(var r in _){_[r].__farm_resource_pot__=filename;global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");