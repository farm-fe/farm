//index.js:
 import { defineExportStar } from "@farm-runtime/module-helper";
import * as zoo_external_all_farm_internal_ from "/external/zoo";
import * as bar_external_all_farm_internal_ from "/external/bar";
const foo = 'foo';
var foo_ts_namespace_farm_internal_ = {
    foo: foo,
    __esModule: true
};
defineExportStar(foo_ts_namespace_farm_internal_, bar_external_all_farm_internal_);
defineExportStar(foo_ts_namespace_farm_internal_, zoo_external_all_farm_internal_);
var test = foo_ts_namespace_farm_internal_.test;
console.log(test, foo);
