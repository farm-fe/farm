# Source Map
Farm supports Source Map, which is automatically enabled by default. It can be enable or disable sourcemap via the `compilation.sourcemap` option.

:::note
Farm will not generate sourcemap for files under node_modules by default, if you want to generate sourcemap for files under node_modules, configure `compilation.sourcemap` to `all`.
:::

Using `compilation.sourcemap` to configuring sourcemap generation:
```ts title="farm.config.ts"
export default {
   compilation: {
     sourcemap: 'all', // generate sourcemap for modules under node_modules
   },
};
```

All options are as below:
* **`true`**: Only generate sourcemap for files not under `node_modules`, and generate a separate sourcemap file
* **`false`**: disable sourcemap
* **`inline`**: Only generate sourcemap for files not under `node_modules`, and inline sourcemap into the product, do not generate a separate file
* **`all`**: generate sourcemap for all files, and generate a separate sourcemap file
* **`all-inline`**: Generate sourcemaps for all files, and inline sourcemaps into the product, do not generate separate files