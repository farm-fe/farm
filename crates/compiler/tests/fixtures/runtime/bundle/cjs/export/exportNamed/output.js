//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};// module_id: dep.ts.farm-runtime
function __commonJs(mod) {
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
var dep_ts_cjs = __commonJs({
    "dep.ts.farm-runtime": (module, exports)=>{
        module.exports.name = 'shulan';
        module.exports.age = 18;
        module.exports.default = 'default';
    }
});
var age$1 = dep_ts_cjs()["age"], name = dep_ts_cjs()["name"];

// module_id: export.ts.farm-runtime
var export_ts_ns = {
    cjsAge: age$1,
    name: name,
    __esModule: true
};

// module_id: runtime.ts.farm-runtime
console.log(export_ts_ns);
const age = 19;
const cjsAge = 20;
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"index.ts":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("index.ts");