//index.js:
 import __farmNodeModule from 'node:module';globalThis.nodeRequire = __farmNodeModule.createRequire(import.meta.url);(globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};import fs$1 from "fs.farm-runtime";
import fs from "node:fs.farm-runtime";
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}

console.log("external 1", fs);

console.log("external 2", fs);

console.log("external 3", fs$1);

(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
import * as __farm_external_module_fs from "fs";import * as __farm_external_module_node_fs from "node:fs";(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"fs": {...__farm_external_module_fs,__esModule:true},"node:fs": {...__farm_external_module_node_fs,__esModule:true}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_7eea.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"632ff088":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _nodefs = _interop_require_default._(farmRequire("node:fs"));
    console.log("external 2", _nodefs.default);
}
,
"9d5a7b13":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _nodefs = _interop_require_default._(farmRequire("node:fs"));
    console.log("external 1", _nodefs.default);
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    farmRequire("9d5a7b13");
    farmRequire("632ff088");
    farmRequire("dea409d9");
}
,
"dea409d9":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _fs = _interop_require_default._(farmRequire("fs"));
    console.log("external 3", _fs.default);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");