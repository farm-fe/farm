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
 // module_id: index.ts
import { __commonJs, _nodeRequire } from "./farm_runtime.js";
import fs from "node:fs";
var index_cjs = __commonJs({
    "index.ts": (module, exports)=>{
        "use strict";
        const os = _nodeRequire('node:os');
        console.log(fs.read, os.cpus);
    }
});
index_cjs();
