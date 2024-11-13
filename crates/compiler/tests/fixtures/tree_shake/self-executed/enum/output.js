//index.js:
 window['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};;((function(){function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
var index_js_cjs = __commonJs((module, exports)=>{
    "use strict";
    console.log('runtime/index.js');
    window['__farm_default_namespace__'].__farm_module_system__.setPlugins([]);
});
index_js_cjs();
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_ecb7.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"569704c1":function  (module, exports, farmRequire, farmDynamicRequire) {
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