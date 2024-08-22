//index.js:
 import __farmNodeModule from 'node:module';globalThis.nodeRequire = __farmNodeModule.createRequire(import.meta.url);(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}import fs$1 from "fs.farm-runtime";
import fs from "node:fs.farm-runtime";
console.log('external 1', fs);

console.log('external 2', fs);

console.log('external 3', fs$1);

(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
import * as __farm_external_module_fs from "fs";import * as __farm_external_module_node_fs from "node:fs";(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"fs": __farm_external_module_fs && __farm_external_module_fs.default && !__farm_external_module_fs.__esModule ? {...__farm_external_module_fs,__esModule:true} : {...__farm_external_module_fs},"node:fs": __farm_external_module_node_fs && __farm_external_module_node_fs.default && !__farm_external_module_node_fs.__esModule ? {...__farm_external_module_node_fs,__esModule:true} : {...__farm_external_module_node_fs}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_7eea.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"632ff088":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_node_fs = module.i(farmRequire('node:fs'));
    console.log('external 2', module.f(_f_node_fs));
}
,
"9d5a7b13":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_node_fs = module.i(farmRequire('node:fs'));
    console.log('external 1', module.f(_f_node_fs));
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    farmRequire("9d5a7b13");
    farmRequire("632ff088");
    farmRequire("dea409d9");
}
,
"dea409d9":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_fs = module.i(farmRequire('fs'));
    console.log('external 3', module.f(_f_fs));
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");