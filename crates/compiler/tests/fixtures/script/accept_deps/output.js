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
})());(function(_){for(var r in _){_[r].__farm_resource_pot__='index_fb79.js';window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"05ee5ec7":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_foo = farmRequire("59ebf907");
    var _f_bar = farmRequire("e185e932");
    _f_foo.foo();
    _f_bar.bar();
    module.meta.hot.accept([
        "foo.js",
        "bar.js"
    ], ([newFooModule, newBarModule])=>{});
}
,
"59ebf907":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "foo", function() {
        return foo;
    });
    function foo() {
        return 'foo';
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_foo = farmRequire("59ebf907");
    farmRequire("05ee5ec7");
    _f_foo.foo();
    ;
}
,
"e185e932":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "bar", function() {
        return bar;
    });
    function bar() {
        return 'bar';
    }
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");