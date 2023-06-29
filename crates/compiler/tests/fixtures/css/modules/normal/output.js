//index.js:
 var entry = function() {
    var __farm_global_this__ = {
        __FARM_TARGET_ENV__: "browser"
    };
    (function(modules, entryModule) {
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
            console.log("runtime/index.js");
            __farm_global_this__.__farm_module_system__.setPlugins([]);
        }
    }, "ec853507");
    (function(modules) {
        for(var key in modules){
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
                "bar": `farm-bar`,
                "main": `farm-main`
            };
        },
        "b5d64806": function(module, exports, farmRequire, dynamicRequire) {
            "use strict";
            Object.defineProperty(exports, "__esModule", {
                value: true
            });
            var _interop_require_default = farmRequire("@swc/helpers/_/_interop_require_default");
            var _indexcss = _interop_require_default._(farmRequire("95fe6ac5"));
            console.log(_indexcss.default);
        }
    });
    var farmModuleSystem = __farm_global_this__.__farm_module_system__;
    farmModuleSystem.bootstrap();
    return farmModuleSystem.require("b5d64806");
}();


//429aa195.css:
  .foo  .farm-hello {
  color: red;
}
.farm-bar {
  color: red;
}
 .farm-main  .description {
  color: blue;
}