//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: reexport.ts
import { default as Foo1 } from "foo1";
import { default as Foo2 } from "foo2";
var reexport_ns = {
    Foo2: Foo2,
    Foo1: Foo1,
    __esModule: true
};

// module_id: index.ts
const Foo1 = '123';
export { default as Foo1 } from "foo1";
export { default as Foo2 } from "foo2";
