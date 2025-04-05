//index.js:
 (function(){const moduleSystem = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(moduleSystem);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "070fbe2d": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
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
    },
    "694da995": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
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
    },
    "6d686e48": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "setFoo", function() {
            return setFoo;
        });
        farmRequire.o(exports, "getFoo", function() {
            return getFoo;
        });
        let foo = 0;
        function setFoo() {
            foo++;
        }
        function getFoo() {
            return foo;
        }
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_inner_side_effect = farmRequire.i(farmRequire("070fbe2d"));
        var _f_import_side_effect = farmRequire.i(farmRequire("fc5423a5"));
        var _f_write_use_side_effect_stmt = farmRequire.i(farmRequire("694da995"));
        console.log(farmRequire.f(_f_inner_side_effect), farmRequire.f(_f_import_side_effect), farmRequire.f(_f_write_use_side_effect_stmt));
    },
    "fc5423a5": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_foo = farmRequire("6d686e48");
        const slot = 'slot';
        const v = _f_foo.setFoo();
        console.log(_f_foo.getFoo());
        exports.default = slot;
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");