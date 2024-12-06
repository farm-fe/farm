//index.js:
 import __farmNodeModule from 'node:module';global.nodeRequire = __farmNodeModule.createRequire(import.meta.url);global['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'node'};function _interop_require_default(obj) {
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
}function __commonJs(mod) {
  var module;
  return () => {
    if (module) {
      return module.exports;
    }
    module = {
      exports: {},
    };
    if(typeof mod === "function") {
      mod(module, module.exports);
    }else {
      mod[Object.keys(mod)[0]](module, module.exports);
    }
    return module.exports;
  };
}var runtime_2_ts_cjs = __commonJs((module, exports)=>{
    "use strict";
    const a = 3;
    const b = 4;
    function BB() {
        const a = 5;
        const b = 6;
        console.log(a, b);
    }
});

runtime_2_ts_cjs();
const a = 3;
const b = 4;
function BB() {
    const a$1 = 5;
    const b$1 = 6;
    console.log(a$1, b$1);
}
console.log(a, b);
const for1 = 'for1';
const for2 = 'for2';
const for3 = 'for3';

const a$2 = 1;
const b$2 = 2;
console.log(a$2, b$2);
{
    const a$3 = 1;
    const b$3 = 2;
}for(var for1$1 in [
    1,
    2,
    3
]){
    console.log(for1$1);
}
for (var for1$1 of [
    1,
    2,
    3
]){
    console.log(for1$1);
}
for (var for2$1 of [
    1,
    2,
    3
]){
    console.log(for2$1);
}
for(var for3$1 = 123; for3$1 < 234; for3$1++){
    console.log(for3$1);
}
for(const for3$2 = 123; for3$2 < 234; for3$2){
    break;
}
global['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
(function(_){var filename = ((function(){return import.meta.url})());for(var r in _){_[r].__farm_resource_pot__=filename;global['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log('aaa');
}
,});global['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);global['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = global['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");