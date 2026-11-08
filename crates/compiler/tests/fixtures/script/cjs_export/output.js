//index.js:
 (function(){if(typeof global==='undefined'||!global['__farm_default_namespace__']||!global['__farm_default_namespace__'].m){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = require("url").pathToFileURL(__filename).href;
        moduleSystem.g(moduleId, module);
    }
})(global["__farm_default_namespace__"].m, {
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        farmRequire.o(exports, "named", function() {
            return named;
        });
        farmRequire.o(exports, "default", function() {
            return foo;
        });
        var named = 'named';
        function foo() {}
    }
});
var __farm_ms__ = global['__farm_default_namespace__'].m;var __farm_entry__=__farm_ms__.r("b5d64806");module.exports=__farm_entry__.default;if(module.exports!==null&&(typeof module.exports==='object'||typeof module.exports==='function')){var __farm_descriptor__=Object.getOwnPropertyDescriptor(module.exports,"named");if(__farm_descriptor__?__farm_descriptor__.configurable:Object.isExtensible(module.exports)){Object.defineProperty(module.exports,"named",{enumerable:true,configurable:true,get:function(){return __farm_entry__["named"]}});}else{module.exports["named"]=__farm_entry__["named"];}}else{module.exports["named"]=__farm_entry__["named"];}