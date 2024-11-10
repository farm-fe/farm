//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function foo() {
    console.log('foo');
}
var dynamic_ts_ns = {
    "default": foo,
    __esModule: true
};

Promise.resolve(dynamic_ts_ns).then((res)=>res.default());
const foo$1 = ()=>Promise.resolve(dynamic_ts_ns);
function loader(m) {}
loader(Promise.resolve(dynamic_ts_ns));
Promise.resolve(dynamic_ts_ns);
const data = {
    foo: Promise.resolve(dynamic_ts_ns)
};
{
    Promise.resolve(dynamic_ts_ns);
}global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");