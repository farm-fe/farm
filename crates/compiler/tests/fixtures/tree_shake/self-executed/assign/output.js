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
    "569704c1": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_b = farmRequire.i(farmRequire("f380ea31"));
        function MapCache(entries) {
            var index = -1, length = entries == null ? 0 : entries.length;
            this.clear();
            while(++index < length){
                var entry = entries[index];
                this.set(entry[0], entry[1]);
            }
        }
        var a = null;
        MapCache.prototype.clear = farmRequire.f(_f_b);
        MapCache.prototype.clear = ()=>(a, farmRequire.f(_f_b));
        exports.default = MapCache;
    },
    "b5d64806": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        var _f_a = farmRequire.i(farmRequire("569704c1"));
        console.log(farmRequire.f(_f_a));
    },
    "f380ea31": function(module, exports, farmRequire, farmDynamicRequire) {
        farmRequire._m(exports);
        function mapCacheClear() {
            this.size = 0;
            this.__data__ = {};
        }
        exports.default = mapCacheClear;
    }
});
var __farm_ms__ = window['__farm_default_namespace__'].m;__farm_ms__.b();var __farm_entry__=__farm_ms__.r("b5d64806");