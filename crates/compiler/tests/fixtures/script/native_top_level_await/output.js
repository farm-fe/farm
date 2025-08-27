//index.js:
 (function(){const __farm_internal_module_system__ = {};
function initModuleSystem() {
    console.log('module-helper.ts');
}
initModuleSystem(__farm_internal_module_system__);
}());(function(moduleSystem, modules) {
    for(var moduleId in modules){
        var module = modules[moduleId];
        module.url = typeof document === "undefined" ? location.href : (document.currentScript && document.currentScript.tagName.toUpperCase() === "SCRIPT" && document.currentScript.src) || location.protocol + "//" + location.host + '/' + "index_dcdc3e0b3362edb8fec2a51d3fa51f8f_js";
        moduleSystem.g(moduleId, module);
    }
})(window["__farm_default_namespace__"].m, {
    "b5d64806": async function(module, exports, farmRequire, farmDynamicRequire) {
        const delay = (ms)=>new Promise((resolve)=>setTimeout(resolve, ms));
        await delay(3000);
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=await __farm_ms__.r("b5d64806");export default __farm_entry__.__esModule && __farm_entry__.default ? __farm_entry__.default : __farm_entry__;