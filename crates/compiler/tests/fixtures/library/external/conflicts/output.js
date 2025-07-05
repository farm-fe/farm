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
import { unresolvedDeep as unresolvedDeepConflict } from '/external/deep/unresolved';
import { existsSync as existsSync$1, existsSync as existsSync$2 } from 'node:fs';
import * as node_fs_external_all_farm_internal_ from "node:fs";
import * as node_module_external_all_farm_internal_ from "node:module";
import * as unresolved_external_all_farm_internal_$1 from "/external/deep/unresolved";
import * as unresolved_external_all_farm_internal_ from "/external/unresolved";
var unresolved = unresolved_external_all_farm_internal_.unresolved;
var unresolvedDeep = unresolved_external_all_farm_internal_$1.unresolvedDeep;
var existsSync = node_fs_external_all_farm_internal_.existsSync;
; // module_id: zoo.ts
var zoo_ts_namespace_farm_internal_ = {
    unresolved: unresolved,
    unresolvedDeep: unresolvedDeep,
    __esModule: true
};
defineExportStar(zoo_ts_namespace_farm_internal_, node_module_external_all_farm_internal_);
; // module_id: bar.ts
console.log('bar existsSync', existsSync$1('bar'));
var bar_ts_namespace_farm_internal_ = {
    existsSync: existsSync,
    unresolved: unresolved,
    unresolvedDeep: unresolvedDeep,
    __esModule: true
};
defineExportStar(bar_ts_namespace_farm_internal_, node_module_external_all_farm_internal_);
; // module_id: foo.ts
console.log('foo existsSync', existsSync$2('foo'));
; // module_id: index.ts
var createRequire = bar_ts_namespace_farm_internal_.createRequire;
console.log('index readFileSync', existsSync('index'));
console.log(unresolved, unresolvedDeep, unresolvedDeepConflict);
console.log(bar_ts_namespace_farm_internal_);
// TODO fix this test
console.log(createRequire);
