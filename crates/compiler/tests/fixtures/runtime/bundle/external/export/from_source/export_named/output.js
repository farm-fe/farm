//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function _mergeNamespaces(n, m) {
    m.forEach(function(e) {
        e && typeof e !== 'string' && !Array.isArray(e) && Object.keys(e).forEach(function(k) {
            if (k !== 'default' && !(k in n)) {
                var d = Object.getOwnPropertyDescriptor(e, k);
                Object.defineProperty(n, k, d.get ? d : {
                    enumerable: true,
                    get: function() {
                        return e[k];
                    }
                });
            }
        });
    });
    return Object.freeze(n);
}
import { readFileSync, readSync } from "node:fs.farm-runtime";
import * as node_fs_ns from "node:fs.farm-runtime";
var exportNamed_ts_ns = _mergeNamespaces({
    readFileSync: readFileSync,
    readSync: readSync,
    __esModule: true
}, [
    node_fs_ns
]);
var readFile = exportNamed_ts_ns["readFile"], writeFileSync = exportNamed_ts_ns["writeFileSync"];

const bundle2A = 'bundle2A';
const bundle2B = 'bundle2B';

var bundle2_index_ts_ns = {
    bundle2A: bundle2A,
    bundle2B: bundle2B,
    __esModule: true
};

global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");