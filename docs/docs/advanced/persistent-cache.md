# Incremental Building

:::tip
Farm supports incremental build by persistent cache since `v0.14.0`
:::

Since `v0.14.0`, Farm supports cache the compiled result to disk, which can greatly speed up the compilation for hot start/hot build. When `persistentCache` is enabled, the compilation time can reduce **up to `80%`**.

Performance compare between cold start(without cache) and hot start(with cache) using [examples/argo-pro](https://github.com/farm-fe/farm/tree/main/examples/arco-pro):


|       | Cold(without cache) | Hot(with cache) | diff        |
| ----- | ------------------- | --------------- | ----------- |
| start | 1519ms              | 371ms           | reduced 75% |
| build | 3582ms              | 562ms           | reduced 84% |


## Using Cache

Using [`compilation.persistentCache`](/docs/config/compilation-options#persistentcache) to `enable/disable` Cache:

```ts
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  compilation: {
    persistentCache: true,
  },
});
```

:::note
`persistentCache: true` is equal to:

```js
({
  persistentCache: {
    // Directory that cache is stored
    cacheDir: "node_modules/.farm/cache",
    // namespace of the cache
    namespace: "farm-cache",
    buildDependencies: [
      "farm.config.ts",
      "@farmfe/core",
      "@farmfe/plugin-react",
      // ... all other dependencies
    ],
    moduleCacheKeyStrategy: {
      timestamp: true,
      hash: true,
    },
  },
});
```
:::

Configuring `persistentCache` to `false` to disable cache.

## Cache Validation

Cache will be validated when trying to reuse it by following conditions, if any of following conditions changed, all cache will be invalidated:


- **Env Object**: configured by `persistentCache.envs`, default to `Farm Env Mode`(`process.env.NODE_ENV`, `process.env.DEV`, `process.env.PROD`), see **[`Environment Variables and Modes`](/docs/features/env)**.
- **lockfile**: If your lockfile changed, means there are dependencies changes, the cache will be invalidated.
* **Build Dependencies**: configured by `persistentCache.buildDependencies`, if any of the buildDependencies changed, all cache will be invalidated.
* **Cache Namespace**: configured by `persistentCache.namespace`, cache under different namespaces won't be reused. If you want to invalidate all cache, you can configure a different namespace.
* **Internal Cache Version**: Farm maintains a cache version internally, if Farm itself changed, for example, render optimization that affects the output between versions of Farm, Farm will bump the cache version and all cache will be invalidated.

If your cache does not work, check out above conditions to figure out the reason. If the cache is broken, you can also delete `node_modules/.farm/cache` to remove cache manually.

## Build Dependencies

Build dependencies is dependencies that can affect the compilation process or compiled output, for examples, plugins or config files. If any of these dependencies changed, all cache will be invalidated.

Build dependencies can be a file path for a package name, for example:

```ts
import { defineConfig } from "@farmfe/core";
import path from "node:path";

export default defineConfig({
  persistentCache: {
    buildDependencies: [
      // a file path
      path.resolve(process.cwd(), "./plugins/my-plugin.js"),
      // a package name, note that this package must expose package.json
      "farm-plugin-custom-xxx",
    ],
  },
});
```

:::note
By default, all config files and its dependencies are included. But if you want to add some additional files or dependencies to invalidate the cache, you can using `buildDependencies` once these files changed, all cache will be invalidated.
:::

## Module Cache Key Strategy

Farm provides 2 strategies to control how to generate module cache key:

- `timestamp`: whether check timestamp of the module, if the update timestamp does not change, the build of this module will be skipped, which has the best performance.
- `hash`: whether check content hash after load and transform, if the content does not change, the left build of this module will be skipped.

By default `timestamp` and `hash` are both enabled.

## Caveats For Plugins

when `timestamp` is enabled, all build stages hooks like `load` and `transform` won't be called. So if the plugin relies `load` and `transform` and it does not implement `plugin_cache_loaded` and `write_plugin_cache` hook, it may not work as expected. For example, if a plugin collect information in `load` and `transform`, all emit them at `finish` hook, it should implement `plugin_cache_loaded` and `write_plugin_cache` hook to load and write cache, otherwise it will not work as expected.

Farm will set `timestamp` to `false` when `output.targetEnv` is `node`.

<!-- ## Dive deep into Persistent  -->
