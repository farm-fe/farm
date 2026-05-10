//index.js:
 import { existsSync } from "node:fs";
import * as node_module_ambiguous_export_all_farm_internal_ from "node:module";
import { unresolved } from "/external/unresolved";
import { unresolvedDeep } from "/external/deep/unresolved";
var node_module_createRequire = node_module_ambiguous_export_all_farm_internal_.createRequire;
; // module_id: zoo.ts
; // module_id: bar.ts
console.log('bar existsSync', existsSync('bar'));
var bar_ts_namespace_farm_internal_ = {
    __esModule: true
};
; // module_id: foo.ts
console.log('foo existsSync', existsSync('foo'));
; // module_id: index.ts
console.log('index readFileSync', existsSync('index'));
console.log(unresolved, unresolvedDeep, unresolvedDeep);
console.log(bar_ts_namespace_farm_internal_);
// TODO fix this test
console.log(node_module_createRequire);
