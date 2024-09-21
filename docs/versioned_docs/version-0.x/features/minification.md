# Minification
Farm supports production minify out of box, which is automatically enabled `in production` by default. It can be enable or disable via the `compilation.minify` option.

Using `compilation.minify` to configure:
```ts title="farm.config.ts"
export default {
   compilation: {
     minify: true
   },
};
```

If minify is enabled:
* for js/ts modules, it will be minified and mangled, all the blank characters will be removed and the variables will be compressed.
* for css and html modules, all spaces will be removed.

:::note
Farm use swc minifier under the hood, all options of swc minifier can be used in Farm.
:::
