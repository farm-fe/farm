//index.js:
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
    "ec853507": function(module, exports, require, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js");
        __farm_global_this__.__farm_module_system__.setPlugins([]);
    }
}, "ec853507");
(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "95fe6ac5": function(module, exports, require, dynamicRequire) {
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
        noop();
        var _default = {
            "hello": `farm-hello`,
            "base": `farm-base`,
            "hide": `farm-hide`,
            "show": `farm-show`
        };
    },
    "b5d64806": function(module, exports, require, dynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        var _interop_require_default = require("@swc/helpers/_/_interop_require_default");
        var _indexcss = _interop_require_default._(require("95fe6ac5"));
        console.log(_indexcss.default.base);
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("b5d64806");


//a1b6e7b5.css:
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