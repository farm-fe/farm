//dep_8b00.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='dep_8b00.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    function _export(target, all) {
        for(var name in all)Object.defineProperty(target, name, {
            enumerable: true,
            get: all[name]
        });
    }
    _export(exports, {
        default: function() {
            return _default;
        },
        dep: function() {
            return dep;
        }
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _dep1 = _interop_require_default._(farmRequire("ef0c4c9d"));
    const dep = "dep";
    function _default() {
        return (0, _dep1.default)();
    }
    console.log("side effect in dep.ts");
}
,
"ef0c4c9d":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return _default;
        }
    });
    function _default() {
        console.log("1111");
    }
}
,});

//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"d2214aaa":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"d2214aaa");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_5d9b.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"7c4a34c2":async function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "default", {
        enumerable: true,
        get: function() {
            return _default;
        }
    });
    var _default = await farmDynamicRequire("05ee5ec7");
}
,
"b5d64806":async function  (module, exports, farmRequire, farmDynamicRequire) {
    const [_interop_require_default__f, _main__f] = await Promise.all([
        farmRequire("@swc/helpers/_/_interop_require_default"),
        farmRequire("7c4a34c2")
    ]);
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = _interop_require_default__f;
    var _main = _interop_require_default._(_main__f);
    console.log(_main.default);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({ '05ee5ec7': [{ path: 'dep_8b00.js', type: 'script' },] });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");