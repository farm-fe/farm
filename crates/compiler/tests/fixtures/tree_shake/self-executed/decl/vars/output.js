//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"0b3bded0":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"0b3bded0");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_ddf1.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        foo1: function() {
            return foo1;
        },
        foo2: function() {
            return foo2;
        },
        foo3: function() {
            return foo3;
        },
        foo4: function() {
            return foo4;
        }
    });
    function foo() {
        console.log("hello world");
    }
    let foo1 = 1, foo2 = 2;
    foo1 = 2;
    var foo3 = foo;
    var foo4 = foo;
    foo3.create = foo;
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _dep = farmRequire("05ee5ec7");
    console.log(_dep.foo1, _dep.foo2, _dep.foo3, _dep.foo4);
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");