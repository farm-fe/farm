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

## Advanced Minify Options

You can pass an object to `minify` for fine-grained control:

```ts title="farm.config.ts"
export default {
  compilation: {
    minify: {
      compress: true,
      mangle: true,
      include: ['src/**'],       // only minify files matching these patterns
      exclude: ['src/vendor/**'], // skip files matching these patterns
    }
  },
};
```

### `include` / `exclude`
Use `include` and `exclude` (regex patterns) to control which modules are minified. By default all modules are minified when minification is enabled.

### `compress` and `mangle`
Both `compress` and `mangle` accept `true`, `false`, or a detailed SWC minifier options object for advanced customization.

:::note
Farm use swc minifier under the hood, refer to [compilation.minify](/docs/config/compilation-options#minify) for detailed options.
:::