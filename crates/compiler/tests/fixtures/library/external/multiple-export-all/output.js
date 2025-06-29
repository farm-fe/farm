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
import * as zoo_external_all_farm_internal_ from "/external/zoo";
import * as bar_external_all_farm_internal_ from "/external/bar";
; // module_id: foo.ts
const foo = 'foo';
var foo_ts_namespace_farm_internal_ = {
    foo: foo,
    __esModule: true
};
defineExportStar(foo_ts_namespace_farm_internal_, bar_external_all_farm_internal_);
defineExportStar(foo_ts_namespace_farm_internal_, zoo_external_all_farm_internal_);
; // module_id: index.ts
var test = foo_ts_namespace_farm_internal_.test;
console.log(test, foo);
