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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_6d6c.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"25593d80":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_b = module.i(farmRequire("f380ea31"));
    module._(exports, "B1", _f_b, "default");
    var _f_a = farmRequire("569704c1");
    module._(exports, "A1", _f_a);
}
,
"569704c1":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "A1", function() {
        return A1;
    });
    function A1() {
        console.log('a1');
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_exportAll = farmRequire("25593d80");
    console.log(_f_exportAll.B1, _f_exportAll.A1);
}
,
"f380ea31":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "default", function() {
        return B1;
    });
    function B1() {
        console.log('b1');
    }
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");