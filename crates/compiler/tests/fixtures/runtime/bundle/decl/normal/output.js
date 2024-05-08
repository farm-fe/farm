//index.js:
 import __farmNodeModule from 'node:module';globalThis.nodeRequire = __farmNodeModule.createRequire(import.meta.url);(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        mod[Object.keys(mod)[0]](module, module.exports);
        return module.exports;
    };
}
var runtime_2_ts_cjs = __commonJs({
    "runtime.2.ts.farm-runtime": (module, exports)=>{
        const a = 3;
        const b = 4;
        function BB() {
            const a = 5;
            const b = 6;
            console.log(a, b);
        }
    }
});

runtime_2_ts_cjs();
const a = 3;
const b = 4;
function BB() {
    const a = 5;
    const b = 6;
    console.log(a, b);
}
console.log(a, b);

const a$1 = 1;
const b$1 = 2;
console.log(a$1, b$1);
(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("aaa");
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry;