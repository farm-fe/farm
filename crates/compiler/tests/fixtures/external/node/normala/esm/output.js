//index.js:
 import __farmNodeModule from 'node:module';globalThis.nodeRequire = __farmNodeModule.createRequire(import.meta.url);(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"0b3bded0":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"0b3bded0");import * as __farm_external_module_jquery from "jquery";(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"jquery": __farm_external_module_jquery && __farm_external_module_jquery.default && !__farm_external_module_jquery.__esModule ? {...__farm_external_module_jquery,__esModule:true} : __farm_external_module_jquery});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _jquery = _interop_require_default._(farmRequire("jquery"));
    console.log(_jquery.default.find);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");