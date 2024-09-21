# 3. 使用 Farm 构建生产项目
默认情况下，Farm 已经为生产构建开启了以下功能的支持：
* **`Tree Shake`**：裁剪和过滤无关模块和代码
* **`压缩`**：对产物进行压缩和混淆，减少产物体积
* **`自动注入 Polyfill`**：Farm 默认对项目降级到 ES5，这意味着 Farm 构建的产物几乎可以在所有浏览器上运行
* **`自动进行局部打包`**: 依据依赖关系以及大小，将项目进行局部打包，对于每次资源请求，生成 25 个左右的资源，在保证并行加载性能的同时，尽可能提升缓存命中率

## 添加 build script
在 `package.json` 中添加 build script:
```json title="package.json" {7}
{
  "name": "1-create-a-project",
  "version": "1.0.0",
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1",
    "start": "farm start",
    "build": "farm build"
  },
  // ... ignore other fields 
}
```
然后执行 `npm run build` 即可。

## 配置 Tree Shake 和压缩
## Configure Tree Shake and compression
* [Tree Shake](/docs/features/tree-shake)
* [Minification](/docs/features/minification)

## 配置局部打包策略
* [Partial Bundling](/docs/features/partial-bundling)