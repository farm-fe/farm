//bundle1.js:
 // module_id: bundle2-foo.ts
import { foo_str1, foo_str2 } from "index.js";
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


//index.js:
 function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}// module_id: foo.ts
import { bundle_str1$1 } from "bundle1.js";
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
export default index_default;
export { index_bar, index_foo$1 as index_foo };
export { foo_str1, foo_str2, foo_default };
