### basic example

> 通常，该字段应包含一个对象， exports 其中每个属性指定模块请求的子路径。对于上述示例，可以使用以下属性： "." for import "package" 和 "./sub/path" for import "package/sub/path" 。以 结尾 / 的属性会将具有此前缀的请求转发到旧的文件系统查找算法。对于以 结尾 _ 的属性， _ 可以采用任何值，并且属性值中的任何值都将替换为取值 \*

```
{
  "exports": {
    ".": "./main.js",
    "./sub/path": "./secondary.js",
    "./prefix/": "./directory/",
    "./prefix/deep/": "./other-directory/",
    "./other-prefix/*": "./yet-another/*/*.js"
  }
}
```

<!-- https://webpack.docschina.org/guides/package-exports/ -->
