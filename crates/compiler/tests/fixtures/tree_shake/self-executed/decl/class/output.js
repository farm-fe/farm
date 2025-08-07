//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "05ee5ec7": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "Foo", function() {
            return Foo;
        });
        class Foo {
        }
        Foo.create = function() {
            return new Validate();
        };
        class Validate {
            constructor(obj, options){
                this.obj = obj;
                this.options = options;
                this.globalConfig = BValidate.globalConfig;
            }
        }
        var BValidate = function(obj, options) {
            return new Validate(obj, Object.assign({
                field: 'value'
            }, options));
        };
        BValidate.globalConfig = {};
        BValidate.setGlobalConfig = function(options) {
            BValidate.globalConfig = options || {};
        };
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_dep = farmRequire("05ee5ec7");
        console.log(_f_dep.Foo);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");