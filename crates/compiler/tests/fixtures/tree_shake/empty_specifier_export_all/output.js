//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"d2214aaa":function(m,e,r,dr){console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
},},"d2214aaa");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_b85a.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
var _export_star = r("@swc/helpers/_/_export_star");
_export_star._(r("ef0c4c9d"), e);
},
"b5d64806":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _export_star = r("@swc/helpers/_/_export_star");
_export_star._(r("05ee5ec7"), e);
var _default = 2;
},
"ef0c4c9d":function(m,e,r,dr){"use strict";
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
    a: function() {
        return a;
    },
    b: function() {
        return b;
    }
});
const a = "1";
const b = "2";
console.log(a, b);
},});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");var a=entry.a;export { a };var b=entry.b;export { b };export default entry.default || entry;