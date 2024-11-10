//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};const a = 'a-runtime.2.ts';
const b = 'b-runtime.2.ts';
function BB() {
    const a$1 = 5;
    const b$1 = 6;
    console.log(a$1, b$1);
}

const a$2 = 'a-runtime.1.ts';
const b$2 = 'b-runtime.1.ts';
function BB$1() {
    const a$3 = 5;
    const b$3 = 6;
    console.log(a$3, b$3);
}
console.log(a$2, b$2, a, b);

const a$4 = 'a-runtime.ts';
const b$4 = 'b-runtime.ts';
console.log(a$4, b$4, a$2, b$2);
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");