//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"d2214aaa":function(m,e,r,dr){console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
},},"d2214aaa");(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"module": {...((globalThis||window||{})['module']||{}),__esModule:true}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_7f1c.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"052dab48":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _default = {
    main: "./main.tsx"
};
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
var _interop_require_default = r("@swc/helpers/_/_interop_require_default");
var _module = r("module");
var _config = r("edceee38");
var _util = _interop_require_default._(r("052dab48"));
var _default = (0, _config.defineFarmConfig)({
    compilation: {
        input: _util.default,
        external: _module.builtinModules
    }
});
},
"edceee38":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "defineFarmConfig", {
    enumerable: true,
    get: function() {
        return defineFarmConfig;
    }
});
function defineFarmConfig(userConfig) {
    return userConfig;
}
},});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;