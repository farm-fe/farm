//index.js:
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
defineExportEsModule(exports);
exportByDefineProperty(exports, "Compiler", ()=>Compiler);
exportByDefineProperty(exports, "Server", ()=>Server);
exportByDefineProperty(exports, "green", ()=>green);
var _f_utils = interopRequireWildcard(require("/external/utils"));
var utils_external_all_farm_internal_ = _f_utils;
var _f_color = interopRequireWildcard(require("/external/color"));
var color_external_all_farm_internal_ = _f_color;
function defineExportStar(to, from) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                value: from[k],
                enumerable: true,
                configurable: true
            });
        }
    });
    return from;
}
; // module_id: server.ts
class Server {
    constructor(){
        green('server constructor');
    }
}
; // module_id: compiler.ts
class Compiler {
    constructor(){
        console.log('Compiler constructor');
    }
}
; // module_id: color.ts
function green(str) {
    console.log('green', str);
}
var color_ts_namespace_farm_internal_ = {
    green: green,
    __esModule: true
};
defineExportStar(color_ts_namespace_farm_internal_, color_external_all_farm_internal_);
defineExportStar(color_ts_namespace_farm_internal_, utils_external_all_farm_internal_);
; // module_id: index.ts
var bold = color_ts_namespace_farm_internal_.bold;
console.log(bold('hello'));
var _f_color1 = require("/external/color");
defineExportStar(exports, _f_color1);
