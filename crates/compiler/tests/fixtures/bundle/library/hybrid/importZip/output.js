//bundle1.js:
 // module_id: bundle2.ts
const bundle2Name = 'bundle2';
const bundle2Age = 18;
function bundle2() {}
export { bundle2, bundle2Age, bundle2Name };


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
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
export { __commonJs, _interop_require_default };


//index.js:
 // module_id: cjs.ts
import { __commonJs, _interop_require_default } from "./farm_runtime.js";
import { bundle2, bundle2Age, bundle2Name } from "./bundle1.js";
import { readFile } from "node:fs";
var cjs_cjs = __commonJs({
    "cjs.ts": (module, exports)=>{
        "use strict";
        Object.defineProperty(exports, "__esModule", {
            value: true
        });
        Object.defineProperty(exports, "default", {
            enumerable: true,
            get: function() {
                return cjs;
            }
        });
        module.exports.cjsName = 'foo';
        module.exports.cjsAge = 18;
        function cjs() {}
    }
});
var cjs = _interop_require_default(cjs_cjs()).default, cjsAge = cjs_cjs()["cjsAge"], cjsName = cjs_cjs()["cjsName"];

// module_id: esm.ts
const esmName = 'esm';
const esmAge = 19;
function esm() {}

// module_id: bar.ts
console.log({
    cjs: {
        cjs: cjs,
        cjsName: cjsName
    },
    readFile: readFile,
    esm: {
        esm: esm,
        esmName: esmName
    },
    bundle2: {
        bundle2: bundle2,
        bundle2Name: bundle2Name
    }
}, 'bar.ts');

// module_id: foo.ts
console.log({
    cjs: {
        cjs: cjs,
        cjsAge: cjsAge
    },
    esm: {
        esm: esm,
        esmAge: esmAge
    },
    bundle2: {
        bundle2: bundle2,
        bundle2Age: bundle2Age
    },
    readFile: readFile
}, 'foo.ts');

// module_id: index.ts
