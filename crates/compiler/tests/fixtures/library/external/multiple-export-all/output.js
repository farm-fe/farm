//index.js:
 import * as zoo_ambiguous_export_all_farm_internal_ from "/external/zoo";
import * as bar_ambiguous_export_all_farm_internal_ from "/external/bar";
var zoo_test = zoo_ambiguous_export_all_farm_internal_.test || bar_ambiguous_export_all_farm_internal_.test;
; // module_id: foo.ts
const foo = 'foo';
; // module_id: index.ts
console.log(zoo_test, foo);
