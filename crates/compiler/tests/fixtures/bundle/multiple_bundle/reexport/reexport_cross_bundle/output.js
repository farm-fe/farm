//bundle1.js:
 // module_id: bundle2-foo.ts
import { foo_str1, foo_str2 } from "./index.js";
const bundle_str1 = 'bundle2 str1';
console.log(bundle_str1);
var bundle2_foo_default = 'bundle2 foo';

// module_id: bundle2.ts
console.log({
    foo_str1: foo_str1,
    foo_str2: foo_str2
});
const bundle_str1$1 = 'bundle str1';
const bundle_str2 = 'bundle str2';
export { bundle_str1$1, bundle_str2 };


//farm_runtime.js:
 // module_id: __FARM_BUNDLE_POLYFILL_SLOT__


//index.js:
 // module_id: foo.ts
import { bundle_str1$1 } from "./bundle1.js";
const foo_str1 = 'foo str1';
const foo_str2 = 'foo str2';
const foo_str3 = 'foo str3';
const index_foo = 234;
console.log(foo_str1, index_foo, foo_str3);
var foo_default = 'foo default';

// module_id: index.ts
console.log(bundle_str1$1);
const index_foo$1 = 'index foo';
const index_bar = 'index bar';
const foo_str1$1 = 123;
const foo_str3$1 = 'index-foo_str3';
console.log(foo_str1$1, foo_str3$1);
var index_default = 'index default';
export { index_bar, index_foo$1 as index_foo, index_default as default };
export { foo_default, foo_str1, foo_str2 };
