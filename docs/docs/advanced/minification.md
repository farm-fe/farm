# Minification
Farm supports production minify out of box, which is automatically enabled `in production` by default. It can be enable or disable via the [`compilation.minify`](/docs/config/compilation-options#minify) option.

```ts title="farm.config.ts"
export default {
   compilation: {
    // enable minification for both development and production
    minify: true
   },
};
```

If minify is enabled:
* For js/ts modules, the code will be `compressed` and `mangled`, and all the blank characters will be removed.
* For css and html modules, all spaces will be removed.

:::note
Farm use swc minifier under the hood, refer to [compilation.minify](/docs/config/compilation-options#minify) for detailed options.
:::