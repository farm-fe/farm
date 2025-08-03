# Lazy Compilation

When comes to a big project, you may want to split them into small pieces and load on demand. This can be achieved by dynamic imports.

```js
const page = React.lazy(() => import("./page")); // lazy load page
```

By default, Farm will lazy compile these dynamic imports in development, only compile them when the module is really executed. Lazy compilation can really speedup the compiling of a large project.

:::note
Lazy Compilation are always disabled for production build.
:::

Note that it is important to use the `dynamic import` properly to make `lazy compilation` work better. For example, if one of your page has a big dependencies, but this dependencies won't be used until this page rendered, then it is necessary to make sure that this big dependencies are dynamic imported, so it won't be compiled util the page rendered.

## Configuring Lazy Compilation

Using `compilation.lazyCompilation` to enable or disable it:

```ts title="farm.config.ts"
import { defineConfig } from "farm";

export default defineConfig({
  compilation: {
    lazyCompilation: true,
  },
});
```

## How Lazy Compilation Work

When lazy compilation is enabled, Farm will analyze all of your `dynamic import` first, for example:

```js
const page = React.lazy(() => import("./page"));
```

Farm will treat `./page` as a module that should be lazy compiled and won't compile it, instead, Farm will return a virtual placeholder module for `./page` like:

```ts
// ... other actions
const compilingModules = FarmModuleSystem.compilingModules;
// return a promise, this promise will be resolved when lazy compilation finished.
let promise = Promise.resolve();

// it has lazy been lazy compiling
if (compilingModules.has(modulePath)) {
  promise = promise.then(() => compilingModules.get(modulePath));
} else {
  // request the dev server for lazy compilation
  const url = "/__lazy_compile?paths=" + paths.join(",") + `&t=${Date.now()}`;
  promise = import(url).then((module: any) => {
    const result: LazyCompileResult = module.default;
    // ...
  });
  // ... more actions
}

export const __farm_async = true;
export default promise;
```

Above example illustrated a basic structure of that virtual placeholder module. When the placeholder executed, it will request the dev server to compile this module and its dependencies. After getting the lazy compiled result from dev server, the placeholder module will patch these changes to Farm's runtime module system.
