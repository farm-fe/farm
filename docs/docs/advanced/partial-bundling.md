# Partial Bundling
`Partial Bundling` is a strategy that Farm uses to bundle modules, similar to what other bundlers do but the goal of Farm's `Partial Bundling` is different.

Unlike other bundlers, Farm will not trying to bundle everything together and then split them out using optimizations like `splitChunks`, on the opposite, Farm will bundle projects into several output files directly. For example, if there are hundreds of modules needed to launch a html page, Farm will try to bundle them into 20-30 output files directly. Farm calls this behavior `Partial Bundling`.

Farm's goal of Partial Bundling is to:
1. **Reduce request numbers and request hierarchy**: Make hundreds or thousands of module requests reduce to 20-30 requests, and avoid loading modules one after one due to dependency hierarchy, which would make resource loading faster.
2. **Increase cache hit rate**: When a modules changed, makes sure that only a few output files are affected, so more cache can be used for a online project.

For traditional bundlers, we may have a hard time to configure complex `splitChunks` or `manualChunks` to achieve the goal above, but in Farm, it is supported natively through `Partial Bundling`.

Note that the default bundling strategy is designed for browser, but it also works well for Node.js. Try [Configuring Partial Bundling](#configuring-partial-bundling) if want to change bundling strategy for Node.js.

:::tip
Refer to [RFC-003 Partial Bundling](https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md) to get more technical details.
:::

## Motivation
There are two main methods of handling modules in web build tools now: Bundling or native ESM. But they both have drawbacks:
* For bundling, bundlers aim to bundle everything together and then split them out for optimization, but splitting is often hard to configure and is hard to balance resources loading performance and cache hit rate manually. 
* For native esm, every module can be compiled, cached separately, but the load performance are heavily affected when there are hundreds of module requests.

So I was always thinking that if there is a strategy to avoid these two extremes - maybe we can do partial bundling? we can just bundle the project into several limited, size balanced resources directly and automatically. I named this thinking `Module Merging` - Find a balance between bundle and unbundled, only bundles a few related modules to improve loading performance without losing cache granularity.

> I renamed `Module Merging` to `Partial Bundling` later because I think `Partial Bundling` can expresses more accurately what I was thinking.

## Partial Bundling Rules
> In this section, we will introduce the basic rules that `Partial Bundling` uses by examples.

First we look into a basic react project example. For a basic react project like below, we only import react and react-dom in the entry script:
```tsx title="index.tsx"
import React from 'react';
import { createRoot } from 'react-dom/client';
import './index.scss';

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(
  <>
    <div>Index page</div>
  </>
);
```
The bundling result will looks like:
```text
./dist/
├── index_9c07.49b83356.js      # contains react-dom
├── index_a35f.0ac21082.js      # contains ./index.tsx
├── index_b7e0.7ab9ca2d.js      # contains react and its dependencies
├── index_ce26.7f833381.css     $ contains ./index.scss
└── index.html                  # contains ./index.html
```
Farm will bundle your project into 5 files by default:
* `2 js files` are from `node_modules` and contains `react`, `react-dom` and their dependencies.
* `1 js file` are from `./index.tsx`
* `1 css file` are from `./index.scss`;
* `1 html file` are from `./index.html`;

Farm uses following rules to get above results:
1. **Mutable and immutable modules should always be in different output files**: By default Farm treat all modules under `node_modules` are immutable, otherwise they are mutable. So `./index.tsx` is in a separate file cause it's a mutable module, so it never be in the same output file with `react` and `react-dom`.
2. **Different type of module are always in different output files**: So `./index.scss` are in a separate file.
3. **Modules in the same package should be in the same output file**: So all `react` modules are always in the same output file, so does `react-dom`.
4. **The target concurrent requests for a resource loading should be between 20-30 by default**: So there are 3 js output files instead of 1 js bundles.
5. **Output files should be of similar size and min resource size should be greater than 20KB by default**: Because `react-dom` is the largest and more than 100KB, it is in a separate file, and `react` and its dependencies are smaller than `20KB`, there are merged into the same output file.

Now we have familiar with `Partial Bundling`'s basic rules, if met problems with partial bundling, using above rules to debug your project. Next we'll cover how to configure partial bundling.

## Configuring Partial Bundling
### Two Configuring Methods
There 2 different ways to control bundling:
* **`groups`**: Tell Farm that you want these modules bundled together as possible, but it's not enforced because of the optimization strategy of Farm. See [Grouping Modules](#grouping-modules) for this method.
* **`enforceResources`**: Tell Farm that you want these module always bundled together, ignore all other optimization strategy constraints. See [Using `enforceResources`](#using-enforceresources) for this method.

### Partial Bundling Options
`Partial Bundling` supports a lot of options to let users customize its behavior. All the options are as below:

1. **`targetConcurrentRequests`**: Farm tries to generate resource numbers as closer as possible to this config value for initial resource loading or a dynamic resource loading.
2. **`targetMinSize`**: The minimum size of generated resources before minify and gzip. Note that `targetMinSize` will not be satisfied if `ModuleBucket's size` is less than `targetMinSize`, `ModuleBucket` will be given priority. Config `enforceTargetMinSize` can be used to enforce size.
3. **`targetMaxSize`**: The maximum size of generated resources before minify and gzip.
4. **`groups`**: A group of modules that should be placed together. Note that this group config is only a hit to the compiler that these modules should be placed together, it may produce multiple resources, if you want to enforce modules in the same resource, you should use `enforceResources`.
    * **name**: Name of this group.
    * **test**: Regex array to match the modules which are in this group.
    * **groupType**: `mutable` or `immutable`, this group only applies to the specified type of modules.
    * **resourceType**: `all`, `initial` or `async`, this group only applies to the specified type of resources.
5. **`enforceResources`**: Array to match the modules that should always be in the same output resource, ignore all other constraints.
    * **name**: Name of this resource.
    * **test**: Regex array to match the modules which are in this resource.
6. **`enforceTargetConcurrentRequests`**: Enforce target concurrent requests for every resource loading, when true, smaller resource will be merged into bigger resource to meet the target concurrent requests. this may cause issue for css resource, be careful to use this option
7. **`enforceTargetMinSize`**: Enforce target min size for every resource, when true, smaller resource will be merged into bigger resource to meet the target concurrent requests. this may cause issue for css resource, be careful to use this option
8. **`immutableModules`**: Regex array to match the immutable modules
9. **`immutableModulesWeight`**: Default to `0.8`, immutable module will have 80% request numbers. For example, if `targetConcurrentRequest` is 25, then immutable resources will take `25 * 80% = 20` by default. This option is to make sure that mutable and immutable modules are isolate, if change your business code, code under node_modules won't be affected.

:::note
In general, you can use `targetConcurrentRequests`, `targetMinSize` and `targetMaxSize` to control the default behavior of Partial Bundling. The default value set by Farm is based on best practice, so make sure it's necessary when you want to change the default value.
:::

### Grouping Modules
you can use `groups` to group modules together, for above basic react project example, using following configuration to make modules under `node_modules` are bundled together:
```ts title="farm.config.ts" {4-9}
export default defineConfig({
  compilation: {
    partialBundling: {
      groups: [
        {
          name: 'vendor-react',
          test: ['node_modules/'],
        }
      ]
    },
  },
});
```
we add a `group item` with `name` and `test` to group `react` and `react-dom` together. The bundle result is:
```
./dist/
├── index_499e.72cf733c.js    # contains `react`, `react-dom` and all other files under node_modules
├── index_a35f.0ac21082.js    # contains `./index.tsx`
├── index_ce26.7f833381.css   # contains `./index.scss`
└── index.html                # contains `./index.html`
```

Now all modules under `node_modules` are bundled into `index_499e.72cf733c.js`. Note that `groups` is not not enforce that all modules matches this group are bundled, a `group` make produce multiple `output file`, because:
1. mutable and immutable module are always in different output files. When both mutable and immutable modules hit this `group`, they will be in different output.
2. when comes to a multi page app or dynamic imported entries, there may be shared modules, and these should modules are always in different output files.

If you need to enforce modules in the same output files, you can use `enforceResources`

### Using `enforceResources`
To group all modules together and ignore all other conditions, you can use `enforceResources`, for example:
```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      enforceResources: [
        {
          name: 'index',
          test: ['.+'],
        }
      ]
      // c-highlight-end
    },
  },
});
```
will produce:
```
./dist/
├── index.7f833381.css  # all css modules are bundled together
├── index.ba5550d9.js   # all script modules are bundled together
└── index.html
```

:::warning
`enforceResources` will ignore all Farm's internal optimization, be careful when you use it.
:::

### Configuring `immutable modules`
Using `immutableModules` to configure immutable modules, by default, Farm set it to `node_modules/`.

```ts title="farm.config.ts"
export default defineConfig({
  compilation: {
    partialBundling: {
      immutableModules: ['node_modules/', '/global-constants']
    },
  },
});
```
Immutable module can affect bundling and incoming persistent cache, be careful if you want to change it.

## Examples
:::note
Normally you don't need to configure bundling manually, if you want to configure the bundles, make sure you really need it. And these examples are only illustrations to help you learn how to configure bundling strategy easily.
:::

### Grouping Files under Same Directory
Grouping `modules` under `src/components` and output them in the same bundle **as possible**. 

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      groups: [
        {
          name: 'components',
          test: ['./src/components'],
        }
      ]
      // c-highlight-end
    },
  },
});
```

### Configuring Bundle Numbers and Size
```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      targetConcurrentRequests: 15,
      targetMinSize: 200 * 1024 // 200 KB
      // c-highlight-end
    },
  },
});
```

In above example, Farm will try to bundle your project into `15` files **as possible**, with min size of each file larger than `200KB` **as possible**.

### Bundle All Modules Together
```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      enforceResources: [
        {
          name: 'index',
          test: ['.+'],
        }
      ]
      // c-highlight-end
    },
  },
});
```
In above example, we enforce to bundle all modules together and ignore all other constraints(for example, request numbers, file size). You can also enforce to bundle some modules together using `enforceResources`:

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    partialBundling: {
      // c-highlight-start
      enforceResources: [
        {
          name: 'index',
          test: ['\\./src/components/.+'],
        }
      ]
      // c-highlight-end
    },
  },
});
```

We enforce to bundle all modules under `src/components` directory.
:::note
`enforceResources` would break internal optimization for bundles, be careful when you use it.
:::