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
    "ec853507": function(module, exports, farmRequire, dynamicRequire) {
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
    "b5d64806": function(module, exports, farmRequire, dynamicRequire) {
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        noop();
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("b5d64806");


//92a5e8c3.css:
 @-webkit-keyframes anim {}
@-moz-keyframes anim {}
@-o-keyframes anim {}
@keyframes anim {}
@-webkit-keyframes anim {
  0% {
    color: red;
  }
}
@-moz-keyframes anim {
  0% {
    color: red;
  }
}
@-o-keyframes anim {
  0% {
    color: red;
  }
}
@keyframes anim {
  0% {
    color: red;
  }
}