//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_ecb7bd149b01fc3bc5090d82beece659_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "569704c1": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "resolveValue", function() {
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
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_a = farmRequire("569704c1");
        console.log(_f_a.resolveValue);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");