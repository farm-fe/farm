//index.js:
 import { existsSync } from "node:fs";
import * as node_module_ambiguous_export_all_farm_internal_ from "node:module";
import { unresolved } from "/external/unresolved";
import { unresolvedDeep } from "/external/deep/unresolved";
var node_module_createRequire = node_module_ambiguous_export_all_farm_internal_.createRequire;
console.log('bar existsSync', existsSync('bar'));
var bar_ts_namespace_farm_internal_ = {
    __esModule: true
};
console.log('foo existsSync', existsSync('foo'));
console.log('index readFileSync', existsSync('index'));
console.log(unresolved, unresolvedDeep, unresolvedDeep);
console.log(bar_ts_namespace_farm_internal_);
console.log(node_module_createRequire);
