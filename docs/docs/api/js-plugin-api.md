# Js Plugin Api

Farm Js Plugin has designed a similar rollup style design plugin system and easy to migrate your plugins/projects from Rollup/Vite/Webpack.

## Configuring Js Plugins

Adding JS plugins by `plugins` option:

```ts title="farm.config.ts" {3,7}
import { defineConfig } from "farm";
// import a js plugin
import farmPluginFoo from "farm-plugin-foo";

export default defineConfig({
  // configuring it in plugins
  plugins: [farmPluginFoo()],
});
```

## Writing Js Plugins

A Farm Js Plugin is a plain javascript object which exposes a set of `hook`s. for example:

```ts title="my-farm-plugin.ts"
// Create a plugin file that exports a plugin function which returns a `JsPlugin` Object:
import type { JsPlugin } from "@farmfe/core";

// Plugin Options
export interface PluginOptions {
  test: boolean;
}
// export a Plugin Function
export default function MyPlugin(options: PluginOptions): JsPlugin {
  // reading options
  const { test } = options;

  // return a object that exposes hook
  return {
    name: "my-farm-plugin",
    // using load hook to load custom modules
    load: {
      filters: {
        resolvedPaths: ["\\.test$"], // filter files to improve performance
      },
      async executor({ resolvedPath }) {
        if (test && resolvedPath.endsWith(".test")) {
          return {
            content: "test file",
            sourceMap: null,
          };
        }
      },
    },
  };
}
```

:::note

- Farm provided `create-farm-plugin` tool to help you create and develop you js plugin quickly. For more details about writing JS plugins, refer to [Writing JS Plugins](/docs/plugins/writing-plugins/js-plugin)
  :::

## Plugin Hook Overview

The Js plugin hook is the same as the Rust plugin, See [Rust Plugin Hook Overview](/docs/api/rust-plugin-api#plugin-hooks-overview).

:::note
Not all hooks are exposed to Js Plugins, only hooks listed in this document are available.
:::

## hooks

### name

- **type: `string`**
- **required: `true`**

The name of this plugins, MUST not be empty.

```ts
export default function MyPlugin() {
  return {
    name: "my-plugin",
    // ...
  };
}
```

### priority

- **type: `number`**
- **required: `false`**
- **default: `100`**

The priority of this plugins, default to `100`. `priority` controls the execution order of plugins, the larger the value, the earlier the plugin is executed.

```ts
export default function MyPlugin() {
  return {
    name: "my-plugin",
    priority: 1000, // make this plugins execute before all other plugins
    // ...
  };
}
```

:::note
Note that the priority of most farm internal plugins like `plugin-script`, `plugin-resolve` is `99`, which means your plugins is always executed before the internal plugins. If your want to make your plugin executed after farm internal plugins, set `priority` to a value that smaller than `99`, for example: `98`. Also the priority value can be negative, you can set it to `-9999` to make sure it is always executed at last.
:::

### config

- **type: `config?: (config: UserConfig) => UserConfig | Promise<UserConfig>;`**
- **hook type: `serial`**
- **required: `false`**

Modify [Farm config](/docs/config/configuring-farm.md) in `config` hook, return the (partial) `modified config`, the returned config will be deeply merged into the config resolved from cli and config file. You can also directly mutate the config.

Example:

```ts
const resolveConfigPlugin = () => ({
  name: "return-resolve-config-plugin",
  config: (_config) => ({
    compilation: {
      resolve: {
        alias: {
          foo: "bar",
        },
      },
    },
  }),
});
```

:::note
`config` hook is called after all `user plugins` are resolved, so add new plugins into the config has no effect.
:::

### configResolved

- **type: `configResolved?: (config: ResolvedUserConfig) => void | Promise<void>;`**
- **hook type: `serial`**
- **required: `false`**

Called when the config resolved(after all plugin's `config` hook being called). Useful when you want to get the final resolved config for your plugin.

Example:

```ts
const myPlugin = () => {
  let farmConfig;

  return {
    name: "my-plugin",
    configResolved(resolvedConfig) {
      // get resolved config
      resolvedConfig = farmConfig;
    },
    transform: {
      filters: {
        moduleTypes: ["js"],
      },
      async executor(param) {
        if (farmConfig.xxx) {
          // ...
        }
      },
    },
  };
};
```

### configureDevServer

- **type: `configureDevServer?: (server: Server) => void | Promise<void>;`**
- **hook type: `serial`**
- **required: `false`**

:::note
Note that this hook runs in development mode only.
:::

Called when `Dev Server` is ready, you can get the dev server instance.

Example:

```ts
const myPlugin = () => {
  let devServer;

  return {
    name: "my-plugin",
    configureDevServer(server) {
      devServer = server;
    },
  };
};
```

:::note
Both `config` and `configResolved` hook of `js plugin` are called before `config` hook of `rust plugin`.
:::

### configureCompiler

- **type: `configureCompiler?: (compiler: Compiler) => void | Promise<void>;`**
- **hook type: `serial`**
- **required: `false`**

Called when `Rust Compiler` is ready, this hook runs in both development and production. You can get `Compiler` instance here

Example:

```ts
const myPlugin = () => {
  let farmCompiler;

  return {
    name: "my-plugin",
    configureCompiler(compiler) {
      farmCompiler = compiler;
    },
  };
};
```

### buildStart

- **type: `buildStart?: { executor: Callback<Record<string, never>, void> };`**
- **hook type: `parallel`**
- **required: `false`**

Called before the compilation starts. You can do some initialization work here.

Example:

```ts
const myPlugin = () => {
  // your plugin operations
  let myPluginContext = createMyPluginContext();

  return {
    name: "my-plugin",
    buildStart: {
      async executor() {
        // set up my plugin before compilation.
        myPluginContext.setup();
      },
    },
  };
};
```

:::note
`buildStart` is only called once for the first compile. Later compiling like `Lazy Compilation` and `HMR Update` won't trigger `buildStart`.
:::

### resolve

- **required: `false`**
- **hook type: `first`**
- **type:**

```ts
type ResolveHook = {
  filters: {
    importers: string[];
    sources: string[];
  };
  executor: Callback<PluginResolveHookParam, PluginResolveHookResult>;
};

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> },
) => Promise<R | null | undefined>;

/// Parameter of the resolve hook
export interface PluginResolveHookParam {
  /// the start location to resolve `source`, being [None] if resolving a entry or resolving a hmr update.
  /// it's id of the parent module, for example: `src/index.ts` or `src/index.vue?vue&type=xxx`
  importer: string | null;
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  kind: ResolveKind;
  /// source of the import. for example in index.ts (import App from "./App.vue")
  /// source should be './App.vue'
  source: string;
}
/// Resolve result of the resolve hook
export interface PluginResolveHookResult {
  /// resolved path, normally a absolute path. you can also return a virtual path, and use [PluginLoadHookResult] to provide the content of the virtual path
  resolvedPath: string;
  /// whether this module should be external, if true, the module won't present in the final result
  external: boolean;
  /// whether this module has side effects, affects tree shaking
  sideEffects: boolean;
  /// the query parsed from specifier, for example, query should be `{ inline: true }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [crate::utils::parse_query] is for you
  query: [string, string][] | null;
  /// meta data of the module, will be passed to [PluginLoadHookParam] and [PluginTransformHookParam]
  meta: Record<string, string> | null;
}
```

:::note
All filters `sources` and `importers` of resolve hook are `regex string`.
:::

Custom `source` resolving from `importer`, for example, resolving `./b` from `a.ts`:

```ts title="a.ts"
import b from "./b?raw";
// ...
```

Then the resolve params would be:

```ts
const param = {
  source: "./b",
  importer: { relative_path: "a.ts", query_string: "" },
  kind: "import",
};
```

The resolve result of default resolver would be:

```rust
const resolve_result = {
  resolved_path: "/root/b.ts",   // resolved absolute path of the module
  external: false, // this module should be included in the final compiled resources and should not be external
  side_effects: false, // this module may be tree shaken as it does not contains side effects
  query: [["raw", ""]], // query from the source.
  meta: {}
}
```

The `HookContext` is used to pass status when you can the hooks recursively, for example, your plugin call `context.resolve` in `resolve hook`:

```ts
const myPlugin = () => ({
  name: "my-plugin",
  resolve: {
    filters: {
      sources: ["^.+foo.+$"],
      importers: ["^src/index.ts$"],
    },
    executor: async (param, context, hookContext) => {
      console.log(param);
      if (hookContext.caller === "my-plugin") {
        return null;
      }
      // replace the original source and resolve new source
      const newSource = param.source.replace("foo", "bar");
      return context.resolve(
        {
          ...param,
          source: newSource,
        },
        {
          caller: "my-plugin",
          meta: {},
        },
      );
    },
  },
});
```

In above example, we call `context.resolve` and pass `caller` as parameter, then we should add a guard like `if (hookContext.caller === 'my-plugin') {` to avoid infinite loop.

Note:

- By default, you `resolve hook` are executed **after** the default resolver inside Farm, only the sources that can not be resolved by internal resolver will be passed to your plugin, which means if you want to override the default resolve, you need to set your **plugin's priority larger** than `101`.
- Usually `resolved_path` is the real absolute path that points to a file. But you can still return a `virtual module id` like `virtual:my-module`, but for virtual module you need to implement `load` hook to custom how to load your virtual module. And in Farm, `resolved_path + query = module_id`.
- `ResolveKind` presents the `import type`, Example values: `require`(imported by commonjs require), `cssImport`(imported by css's import statement), etc.
- `meta` can be shared between plugins and hooks, you can get `meta` from params of `load`, `transform` and `parse` hooks in any plugin.

### load

- **required: `false`**
- **hook type: `first`**
- **type:**

```ts
type LoadHook = {
  filters: {
    importers: string[];
    sources: string[];
  };
  executor: Callback<PluginLoadHookParam, PluginLoadHookResult>;
};

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> },
) => Promise<R | null | undefined>;

export interface PluginLoadHookParam {
  moduleId: string;
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string> | null;
}

export interface PluginLoadHookResult {
  /// the content of the module
  content: string;
  /// the type of the module, for example [ModuleType::Js] stands for a normal javascript file,
  /// usually end with `.js` extension
  moduleType: ModuleType;
  /// source map of the module
  sourceMap?: string | null;
}
```

Custom how to load your module from a resolved module path or module id. For example, load a virtual module:

```ts
const myPlugin = () => ({
  name: "my-plugin",
  load: {
    filters: {
      resolvedPaths: ["^virtual:my-plugin$"],
    },
    executor: async (param, context, hookContext) => {
      if (param.resolvedPath === "virutal:my-plugin") {
        return {
          content: 'export default "foo"',
          moduleType: "js",
        };
      }
    },
  },
});
```

`module_type` and `content` is required when loading modules in your `load` hook. `source_map` is optional, you can return source map if you do transform in the `load` hook(which is not recommended, we recommend to use `transform` hook for this situation) or you load original source map from other locations.

`filters.resolvedPath` of `load hook` is `resolvedPath + query`, for example: `/root/src/index.vue?vue&type=style&lang=css`. If you want to ignore query when filtering modules, you can use `$`: `src/index\\.vue$`; If you want to filter modules by query, for example, filtering `lang=css`, you can use `src/index.vue\\.+\\?vue&.+lang=css`.

### transform

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type TransformHook = {
  filters: {
    importers: string[];
    sources: string[];
  };
  executor: Callback<PluginTransformHookParam, PluginTransformHookResult>;
};

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> },
) => Promise<R | null | undefined>;

export interface PluginTransformHookParam {
  moduleId: string;
  /// source content after load or transformed result of previous plugin
  content: string;
  /// module type after load
  moduleType: ModuleType; // Module Type is 'js' | 'jsx' | 'ts' | 'tsx' | 'css' | 'html'...
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string> | null;
  sourceMapChain: string[];
}

export interface PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  content: string;
  /// you can change the module type after transform.
  moduleType?: ModuleType;
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  sourceMap?: string | null;
  // ignore previous source map. if true, the source map chain will be cleared. and this result should return a new source map that combines all previous source map.
  ignorePreviousSourceMap?: boolean;
}
```

Do transformation based on **`module content`** and **`module type`**. Example for transforming `sass` to `css`:

```ts
export default function farmSassPlugin(
  options: SassPluginOptions = {}
): JsPlugin {
  return {
    name: pluginName,
    load: {
      filters: { resolvedPaths: ['\\.(scss|sass)$'] },
      async executor(param) {
        if (param.query.length === 0 && existsSync(param.resolvedPath)) {
          const data = await readFile(param.resolvedPath);
          return {
            content: data,
            moduleType: 'sass'
          };
        }

        return null;
      }
    },
    transform: {
      filters: {
        moduleTypes: ['sass']
      },
      async executor(param, ctx) {
        const { css: compiledCss, map } = compileSass(param.content);
        return {
          content: compiledCss,
          moduleType: 'css' // transformed sass to css,
          sourceMap: JSON.stringify(map)
          ignorePreviousSourceMap: false,
        }
      }
    }
  }
}
```

Normal steps for writing `transform hook`:

1. add a `if` guard based `moduleType` or `resolvedPath` or `moduleId`
2. do transformation of the `content`
3. return the transformed `content`, `sourceMap` and `moduleType`

For `ignorePreviousSourceMap`, if you handled `param.sourceMapChain` and collapsed the source maps of previous plugins in the `transform hook`. You should set `ignorePreviousSourceMap` to `true` to ensure source map is correct. Otherwise, you should always set this option to `false` and leave source map chain handled by Farm.

For filters:

- When both `resolvedPaths` and `moduleTypes` are specified, take the union.
- `filters.resolvedPaths` is `resolvedPath + query`, for example: `/root/src/index.vue?vue&type=style&lang=css`. If you want to ignore query when filtering modules, you can use `$`: `src/index\\.vue$`; If you want to filter modules by query, for example, filtering `lang=css`, you can use `src/index.vue\\.+\\?vue&.+lang=css`.
- `filters.moduleTypes` is **NOT** `regex`, it must exactly match the `ModuleType` like `css`, `js`, `tsx`, etc.

:::note
`transform` hook is **content to content**. There is a similar hook called `process_module`, `process_module` is **ast to ast**. Js plugin does not support `process_module` hook due to performance issues, if you want **ast to ast** transformations, try [`Rust Plugin`](/docs/plugins/writing-plugins/rust-plugin) instead.
:::

### buildEnd

- **type: `buildEnd?: { executor: Callback<Record<string, never>, void> };`**
- **hook type: `parallel`**
- **required: `false`**

Called after the `ModuleGraph` built, but before the resources render and generation starts. You can do some status updating or finalization work here.

Example:

```ts
const myPlugin = () => {
  // your plugin operations
  let myPluginContext = createMyPluginContext();

  return {
    name: "my-plugin",
    buildEnd: {
      async executor() {
        // update my plugin status
        myPluginContext.updateStatus("module-graph-built");
      },
    },
  };
};
```

:::note
`buildEnd` is only called once for the first compile. Later compiling like `Lazy Compilation` and `HMR Update` won't trigger `buildEnd`.
:::

### renderStart

- **type: `renderStart?: { executor: Callback<Config['config'], void>; };`**
- **hook type: `parallel`**
- **required: `false`**

Called before the resources render starts.

Example:

```ts
const myPlugin = () => {
  // your plugin operations
  let myPluginContext = createMyPluginContext();

  return {
    name: "my-plugin",
    renderStart: {
      async executor() {
        // update my plugin status
        myPluginContext.updateStatus("render-start");
      },
    },
  };
};
```

:::note
`renderStart` is only called once for the first compile. Later compiling like `Lazy Compilation` and `HMR Update` won't trigger `renderStart`.
:::

### renderResourcePot

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type RenderResourcePotHook = JsPluginHook<
  {
    resourcePotTypes?: ResourcePotType[];
    moduleIds?: string[];
  },
  RenderResourcePotParams,
  RenderResourcePotResult
>;

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
) => Promise<R | null | undefined>;
type JsPluginHook<F, P, R> = { filters: F; executor: Callback<P, R> };

export interface RenderResourcePotParams {
  content: string;
  sourceMapChain: string[];
  resourcePotInfo: {
    id: string;
    name: string;
    resourcePotType: ResourcePotType;
    map?: string;
    modules: Record<ModuleId, RenderedModule>;
    moduleIds: ModuleId[];
    data: JsResourcePotInfoData;
    custom: Record<string, string>;
  };
}
export interface RenderResourcePotResult {
  content: string;
  sourceMap?: string;
}
```

`Resource Pot` is the abstract representation of the final output bundle file, you can return transformed `resourcePot content` to mutate the final bundle. For example, rendering css:

```ts
const myPlugin = () => ({
  name: "test-render-resource-pot",
  renderResourcePot: {
    filters: {
      moduleIds: ["^index.ts\\?foo=bar$"],
      resourcePotTypes: ["css"],
    },
    executor: async (param) => {
      return {
        content: param.content.replace("<--layer-->", cssCode),
        sourceMap,
      };
    },
  },
});
```

We transform all `<--layer-->` in css resource pot and replace them to real `css code`.

:::note
When both `filters.moduleIds` and `filters.resourcePotTypes` are specified, take the union.
:::

### augmentResourceHash

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type AugmentResourceHash = JsPluginHook<
  {
    resourcePotTypes?: ResourcePotType[];
    moduleIds?: string[];
  },
  {
    id: string;
    name: string;
    resourcePotType: ResourcePotType;
    map?: string;
    modules: Record<ModuleId, RenderedModule>;
    moduleIds: ModuleId[];
    data: JsResourcePotInfoData;
    custom: Record<string, string>;
  },
  string
>;

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
) => Promise<R | null | undefined>;
type JsPluginHook<F, P, R> = { filters: F; executor: Callback<P, R> };
```

Append resource hash for give Resource Pot. Useful if you want to add additional conditions when generating resource hash.

```ts
const myPlugin = () => ({
  name: "test-augment-resource-pot",
  renderResourcePot: {
    filters: {
      moduleIds: ["^index.ts\\?foo=bar$"],
      resourcePotTypes: ["css"],
    },
    executor: async (param) => {
      return "my-hash-args";
    },
  },
});
```

:::note
When both `filters.moduleIds` and `filters.resourcePotTypes` are specified, take the union.
:::

### finalizeResources

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type FinalizeResourcesHook = {
  executor: Callback<
    FinalizeResourcesHookParams,
    FinalizeResourcesHookParams["resourcesMap"]
  >;
};

export type FinalizeResourcesHookParams = {
  resourcesMap: Record<string, Resource>;
  config: Config["config"];
};

export interface Resource {
  name: string;
  bytes: number[];
  emitted: boolean;
  resourceType: string;
  origin: { type: "ResourcePot" | "Module"; value: string };
  info?: ResourcePotInfo;
}
```

Do some transformations for all generated resources, return `transformed resourcesMap`. You can `add`, `remove`, `modify` final generated resources in this hook.

Note:

- `bytes` is binary of the final output, for `js/css/html` code, you can use `Buffer.from(bytes).toString()` to get the code.
- `name` is the final file name.
- `origin` represent where this `Resource` is from, `ResourcePot` means it's generated from `ResourcePot` which is a modules bundle; `Module` means it's from `Module`, for example, static files like `.png/.jpg` are from `Module`.

### transformHtml

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type TransformHtmlHook = {
  order?: 0 | 1 | 2;
  executor: Callback<{ htmlResource: Resource }, Resource>;
};
```

The `order` is used to configure when to execute `transformHtml` hook:

- `0`: means `pre`, executed before parse and generate resources. You can transform original html in this stage.
- `1` and `2`: means `normal` and `post`, executed after parse and generate resources. In this stage, all `<script>`, `<link>` tag are injected.

Transform the final generated html(after all `<script>`, `<link>` tag are injected).

```ts
const myPlugin = () => ({
  name: "my-plugin",
  transformHtml: {
    order: 2,
    async executor({ htmlResource }) {
      const htmlCode = Buffer.from(htmlResource).toString();

      const newHtmlCode = htmlCode.replace("my-app-data", data);
      htmlResource.bytes = [...Buffer.from(newHtmlCode)];

      return htmlResource;
    },
  },
});
```

:::note
You should modify `bytes` field of `htmlResource` and return the mutated `htmlResource`, mutate any other fields take no effects
:::

### writeResources

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type WriteResourcesHook = {
  executor: (param: FinalizeResourcesHookParams) => void | Promise<void>;
};
```

Called **AFTER** all resources are written to disk.

### pluginCacheLoaded

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type PluginCacheLoadedHook = {
  executor: Callback<number[], undefined | null | void>;
};
```

Extend [`persistent cache`](/docs/advanced/persistent-cache) loading for your plugin.

When `Persistent Cache` enabled, `load` and `transform` hook may be skipped when hitting cache. If your plugin relies on previous compilation result(for example, load a virtual module based on existing modules), you may need to implement this hook to load cached infos of your plugin to ensure cache work as expected.

Example:

```ts
const myPlugin = () => {
  let cachedData;

  return {
    name: "my-plugin",
    pluginCacheLoaded: {
      async executor(bytes) {
        const str = Buffer.from(bytes).toString();
        cachedData = JSON.parse(str);
      },
    },
  };
};
```

:::note
You must decide how to `serialize/deserialize` cache to `bytes` in your plugins. For a basic example, you can deserialize data by `[...Buffer.from(JSON.stringify(data))]`
:::

### writePluginCache

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type WritePluginCacheHook = {
  executor: Callback<undefined, number[]>;
};
```

Extend [`persistent cache`](/docs/advanced/persistent-cache) writing for your plugin. `writePluginCache` is often used together with [pluginCaceLoaded](#plugincacheloaded) to read and write persistent cache for plugin. Return the serialized bytes of your data.

Example:

```ts
const myPlugin = () => {
  let cachedData = { foo: "bar" };

  return {
    name: "my-plugin",
    writePluginCache: {
      async executor() {
        const bytes = [...Buffer.from(JSON.stringify(data))];
        return bytes;
      },
    },
  };
};
```

:::note
You must decide how to `serialize/deserialize` cache to `bytes` in your plugins. For a basic example, you can deserialize data by `[...Buffer.from(JSON.stringify(data))]`
:::

### finish

- **type: `finish?: { executor: Callback<Record<string, never>, void> };`**
- **hook type: `parallel`**
- **required: `false`**

Called before the resources render starts.

Example:

```ts
const myPlugin = () => {
  // your plugin operations
  let myPluginContext = createMyPluginContext();

  return {
    name: "my-plugin",
    finish: {
      async executor() {
        // update my plugin status
        myPluginContext.updateStatus("finish");
      },
    },
  };
};
```

:::note
`finish` is only called once for the first compile. Later compiling like `Lazy Compilation` and `HMR Update` won't trigger `finish`.
:::

### updateModules

- **required: `false`**
- **hook type: `serial`**
- **type:**

```ts
type UpdateModulesHook = {
  executor: Callback<
    { paths: [string, string][] },
    string[] | undefined | null | void
  >;
};
```

Called when calling compiler.update(module_paths). Useful to do some operations like clearing previous state or ignore some files when performing HMR.

- `paths` is paths that will be recompiled for this update
- return the new `paths`, later compilation will update the new returned paths.
