//index.js:
 (globalThis || window || global || self).__farm_namespace__ = '__farm_default_namespace__';(globalThis || window || global || self)[__farm_namespace__] = {__FARM_TARGET_ENV__: 'browser'};(function(modules, entryModule) {
    var cache = {};
    function require(id) {
        if (cache[id]) return cache[id].exports;
        var module = {
            id: id,
            exports: {}
        };
        modules[id](module, module.exports, require);
        cache[id] = module;
        return module.exports;
    }
    require(entryModule);
})({
    "ec853507": function(module, exports, farmRequire, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js")(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setPlugins([]);
    }
}, "ec853507");
(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global || self)[__farm_namespace__].__farm_module_system__.setDynamicModuleResourcesMap({  });(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = (globalThis || window || global || self)[__farm_namespace__];
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "95fe6ac5": function(module, exports, farmRequire, dynamicRequire) {
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
        "";
        var _default = {
            "hello": `farm-hello`,
            "base": `farm-base`,
            "hide": `farm-hide`,
            "show": `farm-show`
        };
    },
    "b5d64806": function(module, exports, farmRequire, dynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
        var _indexcss = _interop_require_default._(farmRequire("95fe6ac5"));
        console.log(_indexcss.default.base);
    }
});
var farmModuleSystem = (globalThis || window || global || self)[__farm_namespace__].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");

//a7dba609.css:
 .farm-base {
  font-size: 20px;
}
.farm-hide {
  display: none;
}
.farm-show {
  display: block;
}
 .farm-hello {
  color: blue;
}