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
import { existsSync } from 'node:fs';
import * as node_fs_external_all_farm_internal_ from "node:fs";
import * as node_module_external_all_farm_internal_ from "node:module";
import * as unresolved_external_all_farm_internal_$1 from "/external/deep/unresolved";
import * as unresolved_external_all_farm_internal_ from "/external/unresolved";
var a = unresolved_external_all_farm_internal_.unresolved;
var b = unresolved_external_all_farm_internal_$1.unresolvedDeep;
var b$1 = node_fs_external_all_farm_internal_.existsSync;
var zoo_ts_namespace_farm_internal_ = {
    a: a,
    b: b,
    __esModule: true
};
defineExportStar(zoo_ts_namespace_farm_internal_, node_module_external_all_farm_internal_);
console.log('bar existsSync', existsSync('bar'));
var bar_ts_namespace_farm_internal_ = {
    a: a,
    b: b$1,
    c: b,
    __esModule: true
};
defineExportStar(bar_ts_namespace_farm_internal_, node_module_external_all_farm_internal_);
console.log('foo existsSync', existsSync('foo'));
var createRequire = bar_ts_namespace_farm_internal_.createRequire;
console.log('index readFileSync', b$1('index'));
console.log(a, b, unresolvedDeepConflict);
console.log(bar_ts_namespace_farm_internal_);
console.log(createRequire);
