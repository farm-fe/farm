# Tree Shake
Farm supports Tree Shake, which is automatically enabled in the default Production environment. It can be turned on or off by the `compilation.treeShake` option.

During Tree Shake, the sideEffects field in package.json will be automatically read, and modules with sideEffects will not perform Tree Shake.

:::note
Farm will treat all circularly dependent modules as sideEffects and will not perform Tree Shake. Please try to avoid circular dependencies in your project.
:::

Tree shake example:
```js title="a.js"
import { b1, b2 } from 'b';
console.log(b1);
```
```js title="b.js"
export b1 = "B1";
export b2 = "B2";
```
`a.js` is entry and it imports `b.js`, after tree shaking, the result is:
```js title="a.js"
import { b1 } from 'b';
console.log(b1);
```
```js title="b.js"
export b1 = "B1";
```
`b2` is not used and will be removed in both `a.js` and `b.js`

## Configuring Tree Shake
Tree Shake is enabled in production mode by default, to disable tree shake, use `compilation.treeShake`:

```ts title="farm.config.ts"
export default {
   compilation: {
     treeShake: false,
   },
};
```

## Deal With Side Effects
When a module contains `side effects`, Farm won't apply tree shake for it, and all of its imported and exports are treated as used. Farm will think following modules have `side effects`:
1. CommonJs modules always have side effects.
2. A module contains `self-executed` statement at global scope has side effects
3. Modules that contains cyclic dependencies has side effects
4. Modules matches `sideEffects` config in its closest `package.json`
5. Entry modules are always has side effects.

Example 1:
```js
const a = require('./')
module.exports = a;
```
CommonJs module are always has side effects.

Example 2:
```js
import a from './';

a();
```
`a()` is executed at global scope and we treat it as side effect.

Example 3:
```js
// a.js
import b from './b.js'

// b.js
import a from './a.js'
```
`a`, `b` are cyclic dependencies, so they will be treated as side effects too.

Example 4:
```json title="package.json"
{
  "name": "my-package",
  "sideEffects": [
    "./global/**.ts"
  ]
}
```
all ts  modules under `global/` are treat as side effects.