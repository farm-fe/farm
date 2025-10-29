//index.js:
 (function(global, factory) {
    typeof exports === 'object' && typeof module !== 'undefined' ? factory(exports, require('node:fs')) : typeof define === 'function' && define.amd ? define([
        'exports',
        'node:fs'
    ], factory) : (global = typeof globalThis !== 'undefined' ? globalThis : global || self, factory(global['__farm_global__'] = {}, global['node:fs']));
})(this, function(exports, __f_umd_node_fs) {
    function exportByDefineProperty(to, to_k, get) {
        if (Object.prototype.hasOwnProperty.call(to, to_k)) {
            return;
        }
        Object.defineProperty(to, to_k, {
            enumerable: true,
            get
        });
    }
    function defineExportEsModule(to) {
        const key = '__esModule';
        if (to[key]) return;
        Object.defineProperty(to, key, {
            value: true
        });
    }
    function getRequireWildcardCache(nodeInterop) {
        if (typeof WeakMap !== "function") return null;
        var cacheBabelInterop = new WeakMap();
        var cacheNodeInterop = new WeakMap();
        return (getRequireWildcardCache = function(nodeInterop) {
            return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
        })(nodeInterop);
    }
    function interopRequireWildcard(obj, nodeInterop) {
        if (!nodeInterop && obj && obj.__esModule) return obj;
        if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
            default: obj
        };
        var cache = getRequireWildcardCache(nodeInterop);
        if (cache && cache.has(obj)) return cache.get(obj);
        var newObj = {
            __proto__: null
        };
        var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
        for(var key in obj){
            if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
                var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
                if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
                else newObj[key] = obj[key];
            }
        }
        newObj.default = obj;
        if (cache) cache.set(obj, newObj);
        return newObj;
    }
    function importDefault(v) {
        if (typeof v.default !== 'undefined') {
            return v.default;
        }
        return v;
    }
    defineExportEsModule(exports);
    exportByDefineProperty(exports, "bar", ()=>bar);
    exportByDefineProperty(exports, "baz", ()=>baz);
    exportByDefineProperty(exports, "foo", ()=>foo);
    exportByDefineProperty(exports, "qux", ()=>qux_js_namespace_farm_internal_);
    var _f_node_fs = interopRequireWildcard(__f_umd_node_fs);
    ; // module_id: qux.js
    const qux = 'QUX';
    var qux_js_namespace_farm_internal_ = {
        qux: qux,
        __esModule: true
    };
    ; // module_id: index.ts
    var foo = importDefault(_f_node_fs) + 1 + _f_node_fs.readFileSync;
    function bar() {
        // try changing this to `foo++`
        // when generating CommonJS
        return foo;
    }
    function baz() {
        return bar();
    }
});
