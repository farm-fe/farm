//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__
function __commonJs(mod) {
    var module;
    return ()=>{
        if (module) {
            return module.exports;
        }
        module = {
            exports: {}
        };
        if (typeof mod === "function") {
            mod(module, module.exports);
        } else {
            mod[Object.keys(mod)[0]](module, module.exports);
        }
        return module.exports;
    };
}
import __farmNodeModule from 'module';
var __nodeRequireInstance = __farmNodeModule.createRequire(import.meta.url);
function _nodeRequire() {
    return __nodeRequireInstance.apply(null, arguments);
}
export { __commonJs, _nodeRequire };


//index.js:
 // module_id: reexport.ts
import { __commonJs, _nodeRequire } from "./farm_runtime.js";
var reexport_cjs = __commonJs({
    "reexport.ts": (module, exports)=>{
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        _export_star(_nodeRequire("node:fs"), exports);
        _export_star(_nodeRequire("node:cluster"), exports);
        const readFile = 123;
        module.exports.name = 123;
    }
});
var Worker = reexport_cjs()["Worker"], readFile = reexport_cjs()["readFile"];

// module_id: foo.ts

// module_id: index.ts
console.log({
    readFile: readFile,
    Worker: Worker
});
export * from "node:cluster";
export * from "node:fs";
