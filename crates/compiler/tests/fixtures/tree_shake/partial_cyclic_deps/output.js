//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"d2214aaa":function(m,e,r,dr){console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
},},"d2214aaa");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_6d6c.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"25593d80":function(m,e,r,dr){"use strict";
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
    A1: function() {
        return _ats.A1;
    },
    B1: function() {
        return _bts.default;
    }
});
var _interop_require_default = r("@swc/helpers/_/_interop_require_default");
var _bts = _interop_require_default._(r("f380ea31"));
var _ats = r("569704c1");
},
"569704c1":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "A1", {
    enumerable: true,
    get: function() {
        return A1;
    }
});
function A1() {
    console.log("a1");
}
},
"b5d64806":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
var _exportAll = r("25593d80");
console.log(_exportAll.B1, _exportAll.A1);
},
"f380ea31":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return B1;
    }
});
function B1() {
    console.log("b1");
}
},});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");