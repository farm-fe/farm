//dep_8b00.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='dep_8b00.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(e, {
    default: function() {
        return _default;
    },
    dep: function() {
        return dep;
    }
});
var _interop_require_default = r("@swc/helpers/_/_interop_require_default");
var _dep1 = _interop_require_default._(r("ef0c4c9d"));
const dep = "dep";
function _default() {
    return (0, _dep1.default)();
}
console.log("side effect in dep.ts");
},
"ef0c4c9d":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function _default() {
    console.log("1111");
}
},});

//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"d2214aaa":function(m,e,r,dr){console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
},},"d2214aaa");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function(m,e,r,dr){"use strict";
dr("05ee5ec7").then((dep)=>{
    console.log(dep);
});
},});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({ '05ee5ec7': [{ path: 'dep_8b00.js', type: 'script' },] });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");