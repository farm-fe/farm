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
})());(function(_){var filename = ((function(){var _documentCurrentScript = typeof document !== "undefined" ? document.currentScript : null;return typeof document === "undefined" ? require("url").pathToFileURL(__filename).href : _documentCurrentScript && _documentCurrentScript.src || new URL("index_98b2.js", document.baseURI).href})());for(var r in _){_[r].__farm_resource_pot__=filename;window['__farm_default_namespace__'].__farm_module_system__.register(r,_[r])}})({"070fbe2d":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    let foo = 0;
    function setFoo() {
        let foo = 0;
        foo++;
    }
    function getFoo() {
        return foo;
    }
    let v = setFoo();
    console.log(getFoo());
    exports.default = {};
}
,
"694da995":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    let foo = 0;
    function setFoo() {
        foo += 1;
    }
    function getFoo() {
        return foo;
    }
    function Bar() {
        console.log('Bar');
    }
    Bar.prototype.foo = setFoo();
    console.log(getFoo());
    exports.default = {};
}
,
"6d686e48":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    module.o(exports, "setFoo", function() {
        return setFoo;
    });
    module.o(exports, "getFoo", function() {
        return getFoo;
    });
    let foo = 0;
    function setFoo() {
        foo++;
    }
    function getFoo() {
        return foo;
    }
}
,
"b5d64806":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_inner_side_effect = module.i(farmRequire("070fbe2d"));
    var _f_import_side_effect = module.i(farmRequire("fc5423a5"));
    var _f_write_use_side_effect_stmt = module.i(farmRequire("694da995"));
    console.log(module.f(_f_inner_side_effect), module.f(_f_import_side_effect), module.f(_f_write_use_side_effect_stmt));
}
,
"fc5423a5":function  (module, exports, farmRequire, farmDynamicRequire) {
    module._m(exports);
    var _f_foo = farmRequire("6d686e48");
    const slot = 'slot';
    const v = _f_foo.setFoo();
    console.log(_f_foo.getFoo());
    exports.default = slot;
}
,});window['__farm_default_namespace__'].__farm_module_system__.setInitialLoadedResources([]);window['__farm_default_namespace__'].__farm_module_system__.setDynamicModuleResourcesMap([],{  });var farmModuleSystem = window['__farm_default_namespace__'].__farm_module_system__;farmModuleSystem.bootstrap();var entry = farmModuleSystem.require("b5d64806");