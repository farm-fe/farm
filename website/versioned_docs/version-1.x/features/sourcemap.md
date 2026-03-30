# Source Map
Farm supports `Source Map`, which is automatically enabled by default. Sourcemap can be `enable` or `disable` via the [`compilation.sourcemap`](/docs/config/compilation-options#sourcemap) option.

:::note
Farm will not generate sourcemap for files under `node_modules` by default, if you want to generate sourcemap for files under node_modules, configure `compilation.sourcemap` to `all`.
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
* **`true`**: Only generate sourcemap for files not under `node_modules`, and generate a **separate sourcemap file**
* **`false`**: disable sourcemap
* **`inline`**: Only generate sourcemap for files not under `node_modules`, and inline sourcemap into the product, do not generate a separate file
* **`all`**: generate sourcemap for all files, and generate a separate sourcemap file
* **`all-inline`**: Generate source maps for all files, and inline source maps into the product, do not generate separate files

:::note
For plugin authors, if you transform the code in [`transform hook`](/docs/api/rust-plugin-api#transform) or [`renderResourcePot hook`](/docs/api/rust-plugin-api#render_resource_pot), you should return the source map of your transformation to ensure source map is correct. Farm maintains a **source map chain** of plugins to trace the final resources back to the real original code.
:::