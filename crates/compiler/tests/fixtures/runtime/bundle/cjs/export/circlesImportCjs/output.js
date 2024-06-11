//index.js:
 import __farmNodeModule from 'node:module';globalThis.nodeRequire = __farmNodeModule.createRequire(import.meta.url);(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function __commonJs(mod) {
  var module;
  return () => {
    if (module) {
      return module.exports;
    }
    module = {
      exports: {},
    };
    mod[Object.keys(mod)[0]](module, module.exports);
    return module.exports;
  };
}var cjs_ts_cjs = __commonJs({
    "cjs.ts.farm-runtime": (module, exports)=>{
        const foo = export_ts_cjs();
        module.exports.name = "foo";
        module.exports.age = 18;
        console.log({
            foo
        });
    }
});

var export_ts_cjs = __commonJs({
    "export.ts.farm-runtime": (module, exports)=>{
        const cjs = cjs_ts_cjs();
        console.log({
            cjs
        });
        module.exports = cjs;
    }
});

export_ts_cjs();
(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");