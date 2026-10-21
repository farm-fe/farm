//index.js:
 var __farmNodeRequire = require;
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
function interopRequireDefault(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
function importDefault(v) {
    if (typeof v.default !== 'undefined') {
        return v.default;
    }
    return v;
}
defineExportEsModule(exports);
exportByDefineProperty(exports, "bar", ()=>bar);
exportByDefineProperty(exports, "default", ()=>index_ts_default);
exportByDefineProperty(exports, "foo", ()=>foo);
var _f_node_fs = interopRequireDefault(require("node:fs"));
; // module_id: index.ts
const os = __farmNodeRequire('node:os');
console.log(importDefault(_f_node_fs).read, os.cpus);
var index_ts_default = {
    read: importDefault(_f_node_fs).read,
    c: 1
};
const foo = 'foo';
const bar = 'bar';
