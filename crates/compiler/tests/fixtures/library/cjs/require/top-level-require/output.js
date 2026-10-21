//index.js:
 function defineExportEsModule(to) {
    const key = '__esModule';
    if (to[key]) return;
    Object.defineProperty(to, key, {
        value: true
    });
}
defineExportEsModule(exports);
var _f_node_module = require("node:module");
; // module_id: index.ts
const require1 = _f_node_module.createRequire(require("node:url").pathToFileURL(__filename).href);
console.log(require1('node:fs'));
