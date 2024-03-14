//index.js:
 (globalThis || window || global)['__farm_default_namespace__'] = {__FARM_TARGET_ENV__: 'browser'};(function(r,e){var t={};function n(r){return Promise.resolve(o(r))}function o(e){if(t[e])return t[e].exports;var i={id:e,exports:{}};r[e](i,i.exports,o,n);t[e]=i;return i.exports}o(e)})({"ec853507":function(m,e,r,dr){console.log("runtime/index.js")(globalThis || window || global)["__farm_default_namespace__"].__farm_module_system__.setPlugins([]);
},},"ec853507");(function(_){for(var r in _){_[r].__farm_resource_pot__='index_ecb7.js';(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"569704c1":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
Object.defineProperty(e, "resolveValue", {
    enumerable: true,
    get: function() {
        return resolveValue;
    }
});
function parse() {
    const mode = 1;
    const typeMap = pathStateMachine[mode];
    return typeMap;
}
const pathStateMachine = [];
pathStateMachine[0] = {
    ["w"]: [
        0
    ],
    ["i"]: [
        3,
        0
    ],
    ["["]: [
        4
    ],
    ["o"]: [
        7
    ]
};
pathStateMachine[1] = {
    ["w"]: [
        1
    ],
    ["."]: [
        2
    ],
    ["["]: [
        4
    ],
    ["o"]: [
        7
    ]
};
pathStateMachine[2] = {
    ["w"]: [
        2
    ],
    ["i"]: [
        3,
        0
    ],
    ["0"]: [
        3,
        0
    ]
};
pathStateMachine[3] = {
    ["i"]: [
        3,
        0
    ],
    ["0"]: [
        3,
        0
    ],
    ["w"]: [
        1,
        1
    ],
    ["."]: [
        2,
        1
    ],
    ["["]: [
        4,
        1
    ],
    ["o"]: [
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
    ["["]: [
        4,
        2
    ],
    ["]"]: [
        1,
        3
    ],
    ["o"]: 8,
    ["l"]: [
        4,
        0
    ]
};
pathStateMachine[5] = {
    ["'"]: [
        4,
        0
    ],
    ["o"]: 8,
    ["l"]: [
        5,
        0
    ]
};
pathStateMachine[6] = {
    ['"']: [
        4,
        0
    ],
    ["o"]: 8,
    ["l"]: [
        6,
        0
    ]
};
function resolveValue() {
    parse();
}
},
"b5d64806":function(m,e,r,dr){"use strict";
Object.defineProperty(e, "__esModule", {
    value: true
});
var _a = r("569704c1");
console.log(_a.resolveValue);
},});(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);(globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap({  });var farmModuleSystem = (globalThis || window || global)['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");