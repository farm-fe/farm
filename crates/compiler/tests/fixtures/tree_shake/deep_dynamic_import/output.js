//dep_8b00.js:
 (function(_){for(var r in _){_[r].__farm_resource_pot__='dep_8b00.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "dep", function() {
        return dep;
    });
    var _f_dep1 = module.i(farmRequire("ef0c4c9d"));
    var dep = 'dep';
    exports.default = function() {
        return module.f(_f_dep1)();
    };
    console.log('side effect in dep.ts');
}
,
"ef0c4c9d":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = function() {
        console.log('1111');
    };
}
,});

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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_5d9b.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"7c4a34c2":async function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    exports.default = await farmDynamicRequire("05ee5ec7");
}
,
"b5d64806":async function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    const [_f_main__f] = await Promise.all([
        farmRequire("7c4a34c2")
    ]);
    var _f_main = module.i(_f_main__f);
    console.log(module.f(_f_main));
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([{ path: 'dep_8b00.js', type: 0 }],{ '05ee5ec7': [0] });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");