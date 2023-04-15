//2a2101a4.js:
 import module from "node:module";
global.__farmNodeRequire = module.createRequire(import.meta.url);
global.__farmNodeBuiltinModules = module.builtinModules;
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
    "../../_internal/runtime/index.js.farm-runtime": function(module, exports, require, dynamicRequire) {
        "use strict";
        console.log("runtime/index.js");
        __farm_global_this__.__farm_module_system__.setPlugins([]);
    }
}, "../../_internal/runtime/index.js.farm-runtime");
(function(modules) {
    for(var key in modules){
        var __farm_global_this__ = globalThis || window || global || self;
        __farm_global_this__.__farm_module_system__.register(key, modules[key]);
    }
})({
    "comp.ts": function(module, exports, require, dynamicRequire) {
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
        require("index.css");
        var _default = "comp";
    },
    "index.css": function(module, exports, require, dynamicRequire) {
        const cssCode = `.body {
  color: red;
}
`;
        const farmId = "index.css";
        const previousStyle = document.querySelector(`style[data-farm-id="${farmId}"]`);
        const style = document.createElement("style");
        style.setAttribute("data-farm-id", farmId);
        style.innerHTML = cssCode;
        if (previousStyle) {
            previousStyle.replaceWith(style);
        } else {
            document.head.appendChild(style);
        }
        module.onDispose(()=>{
            style.remove();
        });
    },
    "index.ts": function(module, exports, require, dynamicRequire) {
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
        var _interopRequireDefault = require("@swc/helpers/lib/_interop_require_default.js").default;
        require("resolved.ts");
        var _comp = _interopRequireDefault(require("comp.ts"));
        console.log(_comp.default);
        var _default = 2;
    },
    "resolved.ts": function(module, exports, require, dynamicRequire) {
        console.log("resolved.ts");
    }
});
var __farm_global_this__ = globalThis || window || global || self;
var farmModuleSystem = __farm_global_this__.__farm_module_system__;
farmModuleSystem.bootstrap();
var entry = farmModuleSystem.require("index.ts").default;
export default entry;
