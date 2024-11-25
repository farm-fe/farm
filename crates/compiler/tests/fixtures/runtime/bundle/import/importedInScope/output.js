//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};const hello = 'hello1';
const hello$1 = 'hello';
console.log(hello, hello$1);
function export_nested() {
    const hello$2 = 'hello';
    console.log(hello$2);
}

var hello$3 = hello;
function say() {
    var hello$4 = hello;
    var hello$1$1 = hello;
    console.log(hello$4);
    function nested_say() {
        var hello$5 = hello;
        var hello$2$1 = hello;
        console.log(hello$5);
    }
}
say();
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");