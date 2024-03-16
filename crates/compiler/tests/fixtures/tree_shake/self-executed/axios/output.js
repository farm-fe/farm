//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"ec853507":function  (module, exports, require, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"ec853507");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_ddf1.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, require, farmDynamicRequire) {
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
    function encode(str) {
        const charMap = {
            "!": "%21",
            "'": "%27",
            "(": "%28",
            ")": "%29",
            "~": "%7E",
            "%20": "+",
            "%00": "\0"
        };
        return encodeURIComponent(str).replace(/[!'()~]|%20|%00/g, function replacer(match) {
            return charMap[match];
        });
    }
    function AxiosURLSearchParams(params, options) {
        this._pairs = [];
        params;
    }
    const prototype = AxiosURLSearchParams.prototype;
    prototype.append = function append(name, value) {
        this._pairs.push([
            name,
            value
        ]);
    };
    prototype.toString = function toString(encoder) {
        const _encode = encoder ? function(value) {
            return encoder.call(this, value, encode);
        } : encode;
        return this._pairs.map(function each(pair) {
            return _encode(pair[0]) + "=" + _encode(pair[1]);
        }, "").join("&");
    };
    var _default = AxiosURLSearchParams;
}
,
"b5d64806":function  (module, exports, require, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = require("@swc/helpers/_/_interop_require_default");
    var _dep = _interop_require_default._(require("05ee5ec7"));
    console.log(_dep.default);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");