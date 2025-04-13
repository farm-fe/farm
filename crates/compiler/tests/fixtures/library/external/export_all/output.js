//index.js:
 import { defineExportStar } from "@farm-runtime/module-helper";
import * as node_module_external_all_farm_internal_ from "node:module";
import * as fs_external_all_farm_internal_ from "fs";
const bar = 'bar';
var bar_ts_namespace_farm_internal_ = {
    bar: bar,
    __esModule: true
};
defineExportStar(bar_ts_namespace_farm_internal_, fs_external_all_farm_internal_);
defineExportStar(bar_ts_namespace_farm_internal_, node_module_external_all_farm_internal_);
export { bar_ts_namespace_farm_internal_ as bar };
