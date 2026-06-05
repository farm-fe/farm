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
import * as fs_ambiguous_export_all_farm_internal_ from "fs";
import * as node_module_ambiguous_export_all_farm_internal_ from "node:module";
; // module_id: bar.ts
const bar = 'bar';
var bar_ts_namespace_farm_internal_ = {
    bar: bar,
    __esModule: true
};
defineExportStar(bar_ts_namespace_farm_internal_, fs_ambiguous_export_all_farm_internal_);
defineExportStar(bar_ts_namespace_farm_internal_, node_module_ambiguous_export_all_farm_internal_);
; // module_id: index.ts
export { bar_ts_namespace_farm_internal_ as bar };
