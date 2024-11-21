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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_2c69.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"10c43cb2":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "registerTickMethod", function() {
        return registerTickMethod;
    });
    const cache = {};
    function registerTickMethod(id, method) {
        cache[id] = method;
    }
}
,
"11ecb1ee":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "scaleFunc", function() {
        return scaleFunc;
    });
    farmRequire("3e3af5b6");
    function scaleFunc() {
        return 'tick';
    }
}
,
"3e3af5b6":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_register = farmRequire("10c43cb2");
    _f_register.registerTickMethod('xxx', ()=>console.log('xxx'));
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "tick", function() {
        return tick;
    });
    var _f_dep_index = farmRequire("11ecb1ee");
    function tick() {
        _f_dep_index.scaleFunc();
    }
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");var tick=entry.tick;export { tick };