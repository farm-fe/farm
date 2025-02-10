/// ```
/// The output should be:
/// ```js
/// // exports of a.js
/// { foo: 1, bar: 2, baz: 3, ns: e_ns, f: 5  }
///
/// // exports of b.js
/// { a: a_ns， foo: 1, bar: 2, baz: 3, ns: e_ns, f: 5, default: 'f_d' }
///
/// // exports of c.js
/// { d: d_ns, bar: 2 }
///
/// // exports of d.js
/// { e: 4, default: 3 }
///
/// // exports of e.js
/// { e: 4, default: function() { console.log('e'); } }
///
/// // exports of f.js
/// { f: 5, default: 'f_d' }
/// ```

import * as a from "./a.js";
import * as b from "./b.js";

console.log(Object.entries(a), Object.entries(b));
