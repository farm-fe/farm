//__farm_runtime.1936680a.mjs:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"d2214aaa":function(m,e,r,dr){console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
},},"d2214aaa");

//debounce_6f74.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='debounce_6f74.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"01609b59":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return debounce;
    }
});
var _utils = r("a5831d05");
function debounce(fn) {
    (0, _utils.debug)("debounce");
    return fn;
}
},});

//index.js:
 import "./__farm_runtime.1936680a.mjs";import "./index_2faa.js";import "./index_64d2.js";(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setExternalModules({"module": {...((globalThis||window||{})['module']||{}),__esModule:true}});(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dcdc.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"b5d64806":function(m,e,r,dr){"use strict";
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
var _module = _interop_require_default._(r("module"));
var _merge = _interop_require_default._(r("726cd210"));
function defineConfig(config) {
    (0, _merge.default)(config, {
        compilation: {
            input: {
                main: "./main.tsx"
            },
            external: _module.default.builtinModules
        }
    });
    return config;
}
dr("01609b59").then((debounce)=>{
    console.log(debounce);
});
var _default = defineConfig({});
},});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources(['index_2faa.js','index_64d2.js']);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({ '01609b59': [{ path: 'debounce_6f74.js', type: 'script' },{ path: 'index_2faa.js', type: 'script' },] });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");export default entry.default || entry;

//index_2faa.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_2faa.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"a5831d05":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "debug", {
    enumerable: true,
    get: function() {
        return debug;
    }
});
function debug(msg) {
    console.log(msg);
}
},});

//index_64d2.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='index_64d2.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"726cd210":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "default", {
    enumerable: true,
    get: function() {
        return merge;
    }
});
var _utils = r("a5831d05");
function merge(a, b) {
    (0, _utils.debug)("merge");
    return a + b;
}
},});