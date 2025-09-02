# 3. 使用 Farm 构建生产项目
默认情况下，Farm 已启用对生产版本的以下功能的支持：
* **`Tree Shake`**：裁剪和过滤不相关的模块和代码
* **`压缩`**：压缩并混淆输出资源，有效减小产物体积。
* **`自动注入Polyfill`**：默认情况下 Farm 降级到现代浏览器(ES7)，如果需要旧版浏览器支持，请配置[`targetEnv`](/docs/config/compilation-options#output-targetenv)
* **`自动局部打包`**：根据依赖关系和大小，对项目进行局部打包。 对于每个资源请求，会生成大约25个资源，以保证并行加载性能，并尽可能提高缓存命中率。

## 配置输出目录
在 `package.json` 中添加构建脚本：

```json title="package.json" {7-8}
{
   "name": "1-create-a-project",
   "version": "1.0.0",
   "scripts": {
     "test": "echo \"Error: no test specified\" && exit 1",
     "start": "farm start",
     "build": "farm build",
     "preview": "farm preview"
   },
   // ...ignore other fields
}
```

然后执行`npm run build`，构建的资源将被生成到`build`目录

```text
build
├─ favicon.ico
├─ index.html
├─ index_02bc.bd68e90b.js
├─ index_02bc.bd68e90b.js.map
├─ index_1c74.4b50f73e.js
├─ index_7734.440d56a3.js
├─ index_880b.4631ecee.js
├─ index_8d49.63f7b906.css
├─ index_8d49.63f7b906.css.map
├─ index_9025.84e1f8e6.js
├─ index_ca37.f2c276ef.js
├─ index_ef2f.e25349d8.js
├─ index_f346.369a7312.js
```

如果您想自定义资源生成的路径，您可以使用：
* [`output.filename`](/docs/config/compilation-options#outputfilename)
* [`output.assetsFilename`](/docs/config/compilation-options#outputassetsfilename)

```ts
import defineConfig from 'farm';

export default defineConfig({
  compilation: {
    output: {
      path: 'build',
      filename: 'assets/[name].[hash].[ext]',
      assetsFilename: 'static/[resourceName].[ext]'
    }
  }
})
```

对于上面的示例，所有`js/css`将被发送到`build/assets/`（例如：`build/assets/index-ea54.abbe3e.js`）。 所有静态资源（例如图像）都将发送到`build/static`（例如：`build/static/background.png`）

## 预览构建的资源
资源构建完成后，您可以通过`npm run Preview`进行预览：

```sh
$ npm run preview

> 3-build@1.0.0 preview
> farm preview

[ Farm ] Using config file at /root/tutorials/3-build-for-production/farm.config.ts
[ Farm ] preview server running at: 

[ Farm ] > Local:   http://localhost:1911/
[ Farm ] > Network: http://198.18.0.1:1911/
[ Farm ] > Network: http://10.242.197.146:1911/
[ Farm ] > Network: http://192.168.1.31:1911/
```

打开`http://localhost:1911/`来预览项目。

## 浏览器兼容性
默认情况下，Farm 将项目构建到本机支持`async/await`的现代浏览器：

* Chrome >= 62
* Firefox >= 63
* Safari >= 13.1
* Edge >= 79

可以使用 [output.targetEnv](/docs/config/compilation-options#output-targetenv) 来配置目标浏览器：

```ts
import { defineConfig } from 'farm';

export default defineConfig({
  compilation: {
    output: {
      targetEnv: 'browser-legacy'
    }
  }
})
```

在上面的例子中，Farm 会将语法降级为 `es5` 并自动注入 polyfill。 然后我们必须安装`core-js@3`来进行`polyfill`注入：

```sh
pnpm add -D core-js@3
```

:::note
* 如果您的目标是旧版浏览器，则需要手动安装 `core-js@3`。
* 如果你想更精确地配置浏览器目标，请参阅[语法 Downgrade 和 Polyfill](/docs/advanced/polyfill)
:::

## 配置 Tree Shake 和 Minify
出于性能原因，像`treeShake`和`minify`这样的生产优化在`development`中默认被`禁用`，而在`生产`中默认被`启用`。 但如果手动配置了`treeShake`或`minify`，则无论`development`或`productive`都将使用默认值。

有关 Tree Shake 和 Minify 的详细信息，请参阅：
* [Tree Shake](/docs/advanced/tree-shake)
* [Minification](/docs/advanced/minification)


## 配置局部打包策略
:::note
详细信息参考[局部打包](/docs/advanced/partial-bundling)。
:::

Farm 已经启用了打包的最佳实践，请确保您确实需要手动配置打包策略，参考[局部打包](/docs/advanced/partial-bundling) 了解详情。