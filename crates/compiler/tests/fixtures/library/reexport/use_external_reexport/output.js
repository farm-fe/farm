//index.js:
 import * as react_dom_external_all_farm_internal_ from "/external/react-dom";
import * as foo_external_all_farm_internal_ from "/external/foo";
import * as node_fs_external_all_farm_internal_ from "node:fs";
var default$1 = node_fs_external_all_farm_internal_.readFile;
var foo = foo_external_all_farm_internal_.foo;
var unstable_batchedUpdates$1 = react_dom_external_all_farm_internal_.unstable_batchedUpdates;
const unstable_batchedUpdates = 123;
console.log({
    unstable_batchedUpdates
});
console.log({
    r1: default$1,
    foo: foo,
    batch: unstable_batchedUpdates$1
});
