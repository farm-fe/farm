# Writing Runtime Plugin

A Farm runtime plugin is a pure javascript object that define a set of hooks to enhance Farm runtime. Example:

```ts
/**
 * HMR client as a Farm Runtime Plugin
 */
import type { Plugin } from "@farmfe/runtime";
import { createHotContext } from "./hot-module-state";
import { HmrClient } from "./hmr-client";

let hmrClient: HmrClient;
// export a Farm runtime plugin object
export default <Plugin>{
  name: "farm-runtime-hmr-client-plugin",
  // define hooks
  bootstrap(moduleSystem) {
    hmrClient = new HmrClient(moduleSystem);
    hmrClient.connect();
  },
  moduleCreated(module) {
    // create a hot context for each module
    module.meta.hot = createHotContext(module.id, hmrClient);
  },
};
```

Above it's a runtime plugin that supports HMR for Farm. Essentials:

- A runtime plugin entry file should **`export`** a default object that defines a set of hooks. e.g `export default <Plugin>{/*...*/}`
- `name` is required to identify the plugin, make sure `name` is unique
- A `hook` is a method that defined in the exported object.

:::note
See [@farmfe/runtime-plugin-hmr](https://github.com/farm-fe/farm/tree/main/packages/runtime-plugin-hmr) for full implementation of above examples.
:::

## Caveat

You should make your runtime plugin as **simple** as possible. You **SHOULD NOT**:

- Use **big dependencies** from node_modules, this would make your farm plugin very large, it's really bad for performance.
- Use new features like `top level await` as these runtime related features are hard to polyfill for low level runtime.

It's really recommended to make sure your runtime plugin **as small and simple as possible**.

:::tip
`import.meta.xxx` will be compiled to `module.meta.xxx`, you can `append values` to `module.meta` in runtime plugins to enhance `import.meta`. For example, `module.meta.hot = createHotContext(module.id, hmrClient)` makes `import.meta.hot` available.
:::

## Conventions

A Farm runtime plugin name should be prefixed by `farm-runtime-plugin`, e.g `farm-runtime-plugin-xxx`.

:::note
Both `plugin.name` and `package name`(Only if you publish your plugin as a package) should be prefixed.
:::

## Using Runtime Plugins

Use `compilation.runtime.plugins` to configure runtime plugins for your project:

```ts
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    runtime: {
      plugins: [
        // relative path
        "./src/my-plugin1.ts",
        // absolute path
        "/root/project/src/my-plugin2.ts",
        // package name
        "@scope/plugin-package-from-node-modules",
      ],
    },
  },
});
```

You can configure runtime plugin item by 3 ways:

- **`relative path`**: Path that is relative to `root`, e.g `./src/my-plugin1.ts` will try load plugin from `<root>/src/my-plugin1.ts`.
- **`absolute path`**: e.g `/root/project/src/my-plugin2.ts`. (Absolute path should be `C:\project\src\my-plugin2.ts` on windows).
- **`package name`**: Farm will try load this package from `node_modules`, e.g `@scope/plugin-package-from-node-modules`.

## Writing Runtime Plugins

:::tip
Farm support loading `.ts` file directly, so you can configure a `.ts` file(or a package whose entry is a `ts` file) in `runtime.plugins` directly.

```ts
export default defineConfig({
  compilation: {
    runtime: {
      plugins: [
        // configuring ts file directly
        "./src/my-plugin.ts",
      ],
    },
  },
});
```

:::

### Create a Plugin

As we mentioned above, a Farm runtime plugin is a pure javascript object that define a set of hooks, you can just create a ts file like:

```ts title="./plugins/runtime.ts"
import type { Plugin } from "@farmfe/runtime";

export default <Plugin>{
  name: "my-plugin",
  // ...
};
```

Then define [hooks](#runtime-plugin-hooks) you need in the exported object:

```ts title="./plugins/runtime.ts"
import type { Plugin } from "@farmfe/runtime";

export default <Plugin>{
  name: "my-plugin",
  moduleCreated(module) {
    // ...
  },
  readModuleCache(module) {
    // ...
  },
  loadResource(resource, targetEnv) {
    // ...
  },
  // ... more hooks as long as you need
};
```

### Debug the Plugin

Configure the plugin you created in `runtime.plugins`:

```ts
export default defineConfig({
  compilation: {
    runtime: {
      plugins: ["./plugins/runtime.ts"],
    },
  },
});
```

Then start the Farm project, this plugin will be injected in the runtime of output resources.

### Publish the Plugin(Optional)

You can publish the runtime plugin to npm registry to share your Farm runtime plugin. Just create a `package.json` like:

```json
{
  "name": "@farmfe/runtime-plugin-hmr",
  "version": "3.4.2",
  "description": "Runtime hmr plugin of Farm",
  // c-highlight-start
  "main": "src/index.ts"
  // c-highlight-end
  // ... ignore other fields
}
```

You can just export `ts` file using `"main": "src/index.ts"`.

## Runtime Plugin Hooks

See [Runtime Plugin API](/docs/api/runtime-plugin-api)
