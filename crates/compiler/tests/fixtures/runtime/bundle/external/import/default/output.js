//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};import fs$1 from "fs.farm-runtime";
import fs from "node:fs.farm-runtime";
console.log('external 1', fs);

console.log('external 2', fs);

console.log('external 3', fs$1);

global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
import * as __farm_external_module_fs from "fs";import * as __farm_external_module_node_fs from "node:fs";global['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"fs": __farm_external_module_fs && __farm_external_module_fs.default && !__farm_external_module_fs.__esModule ? {...__farm_external_module_fs,__esModule:true} : {...__farm_external_module_fs},"node:fs": __farm_external_module_node_fs && __farm_external_module_node_fs.default && !__farm_external_module_node_fs.__esModule ? {...__farm_external_module_node_fs,__esModule:true} : {...__farm_external_module_node_fs}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_7eea.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"632ff088":function  (module, exports, farmRequire, farmDynamicRequire) {
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
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");