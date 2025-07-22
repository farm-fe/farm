//index.js:
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
import { existsSync } from "node:fs";
import * as node_module_ambiguous_export_all_farm_internal_ from "node:module";
import { unresolved } from "/external/unresolved";
import { unresolvedDeep, unresolvedDeep as unresolvedDeepConflict } from "/external/deep/unresolved";
var node_module_createRequire = node_module_ambiguous_export_all_farm_internal_.createRequire;
; // module_id: zoo.ts
; // module_id: bar.ts
console.log('bar existsSync', existsSync('bar'));
var bar_ts_namespace_farm_internal_ = {
    __esModule: true
};
defineExportStar(bar_ts_namespace_farm_internal_, node_module_ambiguous_export_all_farm_internal_);
; // module_id: foo.ts
console.log('foo existsSync', existsSync('foo'));
; // module_id: index.ts
console.log('index readFileSync', existsSync('index'));
console.log(unresolved, unresolvedDeep, unresolvedDeepConflict);
console.log(bar_ts_namespace_farm_internal_);
// TODO fix this test
console.log(node_module_createRequire);
