# Tree Shake

Farm 支持 Tree Shake，该功能在默认的生产环境中自动启用，可以通过[compilation.treeShaking](/zh/document/config/compcompation-option#treeshaking)选项打开或关闭。

在 Tree Shake 期间，package.json 中的 sideEffects 字段将被自动读取，具有 sideEffects 的模块将不会执行 Tree Shake。

:::note
Farm 会将所有循环依赖的模块视为 sideEffects，不会执行 Tree Shake。请尽量避免项目中的循环依赖。
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

`a.js` 是 entry，它导入`b.js`，tree shaking 后，结果是：

```js title="a.js"
import { b1 } from 'b';
console.log(b1);
```

```js title="b.js"
export b1 = "B1";
```

`b2`未使用，将在`a.js`和 `b.js`中删除

## 配置 Tree Shake

Tree Shake 默认在生产模式下启用，要禁用 tree Shake，请使用 `compilation. treeShake`：

```ts title="farm.config.ts"
export default {
  compilation: {
    treeShake: false,
  },
};
```

## 处理 Side Effects

当一个模块包含`Side Effects`时，Farm 不会为其应用 tree shake，其所有导入和导出都被视为已使用。Farm 会认为以下模块有 `Side effects`：

1. Common Js 模块总是有副作用.
2. 一个模块包含`自执行`的语句在全局范围有副作用
3. 包含循环依赖的模块有副作用
4. 模块匹配 `sideEffects` 配置在其最接近的 `package. json`
5. 入口模块总是有副作用

示例 1:

```js
const a = require('./');
module.exports = a;
```

Common Js 模块总是有副作用.

示例 2:

```js
import a from './';

a();
```

`a()` 在全局范围内执行，我们将其视为副作用.

Example 3:

```js
// a.js
import b from './b.js';

// b.js
import a from './a.js';
```

`a`，`b`是循环依赖关系，所以它们也会被视为副作用.

Example 4:

```json title="package.json"
{
  "name": "my-package",
  "sideEffects": ["./global/**.ts"]
}
```

所有 `global/` 下的 ts 模块都被视为副作用.
