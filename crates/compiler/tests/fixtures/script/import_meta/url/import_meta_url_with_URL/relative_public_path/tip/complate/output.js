//foo_bar-47689c.txt:
 foo_bar

//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_dff7f80fb115803750625cd13575beb5_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b334ec2f": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        exports.default = "/foo_bar-47689c.txt";
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_foo_bar = farmRequire.i(farmRequire("b334ec2f"));
        var _f_foo_bar1 = farmRequire.i(farmRequire("b334ec2f"));
        var _f_foo_bar2 = farmRequire.i(farmRequire("b334ec2f"));
        const path1 = 'foo';
        const bar = 'bar';
        new URL({
            "./foo/bar/foo_bar.txt": farmRequire.f(_f_foo_bar2)
        }[`./foo/${path1}/${bar}`], module.meta.url);
        new URL({}[`./foo/${path1}-${bar}`], module.meta.url);
        new URL({
            "./foo/bar/foo_bar.txt": farmRequire.f(_f_foo_bar1)
        }[`./foo/${path1}/**/${bar}`], module.meta.url);
        new URL({
            "./foo/bar/foo_bar.txt": farmRequire.f(_f_foo_bar)
        }["./foo/**/*/**"], module.meta.url);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;