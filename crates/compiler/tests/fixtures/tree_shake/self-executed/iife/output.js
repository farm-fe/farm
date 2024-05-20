//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"ec853507":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"ec853507");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_ecb7.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"569704c1":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    (function() {
        const iife_foo = "iife-foo";
    })();
    var UrlType;
    (function(UrlType) {
        UrlType[UrlType["Empty"] = 1] = "Empty";
        UrlType[UrlType["Hash"] = 2] = "Hash";
        UrlType[UrlType["Query"] = 3] = "Query";
        UrlType[UrlType["RelativePath"] = 4] = "RelativePath";
        UrlType[UrlType["AbsolutePath"] = 5] = "AbsolutePath";
        UrlType[UrlType["SchemeRelative"] = 6] = "SchemeRelative";
        UrlType[UrlType["Absolute"] = 7] = "Absolute";
    })(UrlType || (UrlType = {}));
    function _default() {
        console.log("foo");
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _a = _interop_require_default._(farmRequire("569704c1"));
    console.log(_a.default);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");