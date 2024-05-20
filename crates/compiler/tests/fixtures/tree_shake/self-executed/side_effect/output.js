//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};t[e]=i;r[e](i,i.exports,o,n);return i.exports}o(e)})({"ec853507":function  (module, exports, farmRequire, farmDynamicRequire) {
    console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
}
,},"ec853507");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_98b2.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"070fbe2d":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    let foo = 0;
    function setFoo() {
        let foo = 0;
        foo++;
    }
    function getFoo() {
        return foo;
    }
    let v = setFoo();
    console.log(getFoo());
    var _default = {};
}
,
"694da995":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    let foo = 0;
    function setFoo() {
        foo += 1;
    }
    function getFoo() {
        return foo;
    }
    function Bar() {
        console.log("Bar");
    }
    Bar.prototype.foo = setFoo();
    console.log(getFoo());
    var _default = {};
}
,
"6d686e48":function  (module, exports, farmRequire, farmDynamicRequire) {
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
        getFoo: function() {
            return getFoo;
        },
        setFoo: function() {
            return setFoo;
        }
    });
    let foo = 0;
    function setFoo() {
        foo++;
    }
    function getFoo() {
        return foo;
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
    var _inner_side_effect = _interop_require_default._(farmRequire("070fbe2d"));
    var _import_side_effect = _interop_require_default._(farmRequire("fc5423a5"));
    var _write_use_side_effect_stmt = _interop_require_default._(farmRequire("694da995"));
    console.log(_inner_side_effect.default, _import_side_effect.default, _write_use_side_effect_stmt.default);
}
,
"fc5423a5":function  (module, exports, farmRequire, farmDynamicRequire) {
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
    var _foo = farmRequire("6d686e48");
    const slot = "slot";
    const v = (0, _foo.setFoo)();
    console.log((0, _foo.getFoo)());
    var _default = slot;
}
,});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");