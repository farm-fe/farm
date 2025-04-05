//index.js:
 import { existsSync as existsSync$1, existsSync as existsSync$2 } from 'node:fs';
import * as node_fs_external_namespace_farm_internal_ from "node:fs";
import * as node_module_external_namespace_farm_internal_ from "node:module";
var existsSync = node_fs_external_namespace_farm_internal_.existsSync;
console.log('bar existsSync', existsSync$1('bar'));
var bar_ts_namespace_farm_internal_ = {
    existsSync: existsSync,
    external_all_farm_internal_: node_module_external_all_farm_internal_,
    __esModule: true
};
console.log('foo existsSync', existsSync$2('foo'));
console.log('index readFileSync', existsSync('index'));
console.log(bar_ts_namespace_farm_internal_);
console.log(createRequire);
