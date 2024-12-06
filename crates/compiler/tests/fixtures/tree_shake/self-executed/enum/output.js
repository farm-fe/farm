//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}function __commonJs(mod) {
  var module;
  return () => {
    if (module) {
      return module.exports;
    }
    module = {
      exports: {},
    };
    if(typeof mod === "function") {
      mod(module, module.exports);
    }else {
      mod[Object.keys(mod)[0]](module, module.exports);
    }
    return module.exports;
  };
}((function(){var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
index_js_cjs();
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_ecb7.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"569704c1":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "resolveValue", function() {
        return resolveValue;
    });
    function parse() {
        const mode = 1;
        const typeMap = pathStateMachine[mode];
        return typeMap;
    }
    const pathStateMachine = [];
    pathStateMachine[0] = {
        ['w']: [
            0
        ],
        ['i']: [
            3,
            0
        ],
        ['[']: [
            4
        ],
        ['o']: [
            7
        ]
    };
    pathStateMachine[1] = {
        ['w']: [
            1
        ],
        ['.']: [
            2
        ],
        ['[']: [
            4
        ],
        ['o']: [
            7
        ]
    };
    pathStateMachine[2] = {
        ['w']: [
            2
        ],
        ['i']: [
            3,
            0
        ],
        ['0']: [
            3,
            0
        ]
    };
    pathStateMachine[3] = {
        ['i']: [
            3,
            0
        ],
        ['0']: [
            3,
            0
        ],
        ['w']: [
            1,
            1
        ],
        ['.']: [
            2,
            1
        ],
        ['[']: [
            4,
            1
        ],
        ['o']: [
            7,
            1
        ]
    };
    pathStateMachine[4] = {
        ["'"]: [
            5,
            0
        ],
        ['"']: [
            6,
            0
        ],
        ['[']: [
            4,
            2
        ],
        [']']: [
            1,
            3
        ],
        ['o']: 8,
        ['l']: [
            4,
            0
        ]
    };
    pathStateMachine[5] = {
        ["'"]: [
            4,
            0
        ],
        ['o']: 8,
        ['l']: [
            5,
            0
        ]
    };
    pathStateMachine[6] = {
        ['"']: [
            4,
            0
        ],
        ['o']: 8,
        ['l']: [
            6,
            0
        ]
    };
    function resolveValue() {
        parse();
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_a = farmRequire("569704c1");
    console.log(_f_a.resolveValue);
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");