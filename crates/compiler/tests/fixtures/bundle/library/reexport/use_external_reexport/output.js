//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: reexport.ts
import { foo } from "foo";
import { readFile } from "node:fs";
import { unstable_batchedUpdates as unstable_batchedUpdates$1 } from "react-dom";

// module_id: index.ts
const unstable_batchedUpdates = 123;
console.log({
    unstable_batchedUpdates: unstable_batchedUpdates
});
console.log({
    r1: readFile,
    foo: foo,
    batch: unstable_batchedUpdates$1
});
