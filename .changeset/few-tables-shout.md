---
"@farmfe/core": minor
---

Support tree shake `import * as ns from './xxx'`. `./xxx` can be tree-shaken if following rules are met:
- `ns` is used as member prop, example: `ns.a`
- `ns` is used as member literal computed, example: `ns['a']` 

For example:

```ts
// b.ts
export const a = 1
export const b = 2

// a.ts
import * as ns from './b'

console.log(ns.a)
console.log(ns['a'])
```

After tree shaking, the result will be:
```ts
// b.ts
export const a = 1 // a is preserved and b is removed.

// a.ts
import * as ns from './b'
console.log(ns.a)
console.log(ns['a'])
```

But if `ns` is met rules above, then all the fields will be preserved, example:
```ts
// b.ts
export const a = 1
export const b = 2

// a.ts
import * as ns from './b'

console.log(ns)
```

After tree shaking, the result will be:
```ts
// b.ts
export const a = 1 // both a and b are preserved
export const b = 2
// a.ts
import * as ns from './b'
console.log(ns.a)
console.log(ns['a'])
```
