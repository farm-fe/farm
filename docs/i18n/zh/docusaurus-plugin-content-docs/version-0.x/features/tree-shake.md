# Tree Shake
Farm 支持 Tree Shake，在默认 Production 环境下自动开启。通过 `compilation.treeShake` 选项可控制开启或者关闭。

Tree Shake 时，会自动读取 package.json 中的 sideEffects 字段，有 sideEffect 的模块将不会进行 Tree Shake。

:::note
Farm 会将所有循环依赖的模块视作 sideEffect，不会进行 Tree Shake，请尽量避免项目中存在循环依赖。
:::

Tree shake 示例:
```js title="a.js"
import { b1, b2 } from 'b';
console.log(b1);
```
```js title="b.js"
export b1 = "B1";
export b2 = "B2";
```
`a.js` 是入口，它导入了 `b.js`，经过 Tree Shaking，结果是：
```js title="a.js"
import { b1 } from 'b';
console.log(b1);
```
```js title="b.js"
export b1 = "B1";
```
`b2`未使用，将在`a.js`和`b.js`中删除

## 配置 Tree Shake
默认情况下，在生产模式下启用 Tree Shake，要禁用 Tree Shake，请使用`compilation.treeShake`：

```ts title="farm.config.ts"
export default {
   compilation: {
     treeShake: false,
   },
};
```

## 处理 Side Effects
当模块包含`副作用`时，Farm 不会对其应用 tree shake，并且其所有导入和导出都将被视为已使用。 Farm 会认为以下模块有`副作用`：
1. CommonJs 模块总是有副作用。
2. 模块在全局范围内包含`自执行`语句有副作用
3. 包含循环依赖的模块有副作用
4. 模块与最接近的 `package.json` 中的 `sideEffects` 配置相匹配
5.入口模块总是有副作用。

Example 1:
```js
const a = require('./')
module.exports = a;
```
CommonJs 模块总是有副作用。

Example 2:
```js
import a from './';

a();
```
`a()` 在全局范围内执行，我们将其视为副作用。

Example 3:
```js
// a.js
import b from './b.js'

// b.js
import a from './a.js'
```
`a`、`b` 是循环依赖，因此它们也将被视为副作用。

Example 4:
```json title="package.json"
{
  "name": "my-package",
  "sideEffects": [
    "./global/**.ts"
  ]
}
```
`global/` 下的所有 ts 模块都被视为副作用。