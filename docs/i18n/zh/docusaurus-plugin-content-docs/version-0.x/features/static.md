# 静态资源
> v0.4 及以上支持
Farm 支持三种资源加载方式： `url` , `inline` , `raw` 。

## 以 URL 形式使用
导入图片：
```jsx
import rocketUrl from './assets/rocket.svg'; // return the url of this image

export function Main() {
  return <img src={rocketUrl} /> // using the url
}
```

导入图片时默认以 URL 的形式。 当使用 URL 形式导入图像时，图像将直接复制到输出目录，并且图像模块本身将被编译为 js 模块，如下所示：

```js
export default '/rocket.<content hash>.svg'
```

使用 `compilation.output.assetsFilename` 来配置你的资源名称。

## 内联

使用查询 `?inline` 告诉 Farm 你想要内联你的资源，然后资源将被转换为 `base64`，例如：

```js
// importer
import logo from './assets/logo.png?inline'; // logo is a base 64 str

// the image module will be compiled to:
export default 'data:image/png,base64,xxxxx==';
```

## 原始字符串
例如，使用查询`?raw`告诉 Farm 您要读取资产的原始字符串

```js
// import 
import logo from './assets/license.txt?raw'; // return the content string of the assets

// the txt file will be compiled to:
export default 'MIT xxxx';
```

## 相关配置
* 使用`compilation.output.assetFileName`来控制生产文件名
* 使用`compilation.assets.include`将更多类型的文件视为资产模块。

```js
export default {
  compilation: {
    output: {
      assetsFilename: 'assets/[resourceName].[hash].[ext]', // [] 里面的是 Farm 支持的全部占位符
    },
    assets: {
      include: ['txt'] // 额外静态资源类型
    }
  }
}
```