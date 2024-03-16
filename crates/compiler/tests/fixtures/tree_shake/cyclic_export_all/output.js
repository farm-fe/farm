//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"d2214aaa":function  (module, exports, require, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"d2214aaa");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_dd58.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"569704c1":function  (module, exports, require, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _export_star = require("@swc/helpers/_/_export_star");
    _export_star._(require("f380ea31"), exports);
}
,
"b5d64806":function  (module, exports, require, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _a = require("569704c1");
    console.log(_a.c1);
}
,
"c23f7b06":function  (module, exports, require, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "d1", {
        enumerable: true,
        get: function() {
            return d1;
        }
    });
    const d1 = 3;
}
,
"f06623f5":function  (module, exports, require, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    Object.defineProperty(exports, "c1", {
        enumerable: true,
        get: function() {
            return c1;
        }
    });
    require("f380ea31");
    const c1 = 1;
}
,
"f380ea31":function  (module, exports, require, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _export_star = require("@swc/helpers/_/_export_star");
    _export_star._(require("f06623f5"), exports);
    var _d = require("c23f7b06");
    console.log(_d.d1);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");