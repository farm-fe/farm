# Rust Plugin Api
:::note
This document only covers the details of the plugin hooks. For how to create, build and publish a rust plugin see: [Writing Rust Plugins](/docs/plugins/writing-plugins/rust-plugin)
:::

## Configuring Rust Plugins

Adding Rust plugins by `plugins` option:

```ts title="farm.config.ts" {3,7}
import { defineConfig } from "farm";

export default defineConfig({
  // configuring it in plugins
  plugins: [
    ['@farmfe/plugin-sass', { /** plugin options here */ }]
  ],
});
```

Configuring the Rust plugin package name(or path) in string and its options in object.

## Writing Rust Plugin
See [Writing Rust Plugins](/docs/plugins/writing-plugins/rust-plugin) for details.


## Plugin Hooks Overview
Farm provides a lot of rollup-style hooks, these hooks are divided into build stage and generate stage:
![Farm Plugin Hooks](/img/farm-plugin-hooks.png)

All plugin hooks accept a parameter called [`CompilationContext`](https://docs.rs/farmfe_core/latest/farmfe_core/context/struct.CompilationContext.html). All of the shared compilation info are stored in the `context`.

There are three kinds of hooks (the same as Rollup):

* `first`: The hooks execute in serial and return immediately when a hook returns a non-null value. (The null means null and undefined in JS, None in Rust).
* `serial`: The hooks execute in serial, and every hook's result will pass to the next hook, using the last hook's result as the final result.
* `parallel`: The hooks execute in parallel in a thread pool and should be isolated.

:::note
For full `Plugin Hooks` signature, see [Plugin Trait](https://docs.rs/farmfe_core/latest/farmfe_core/plugin/trait.Plugin.html)
:::

## name
- **required: `true`**
- **default:**
```rust
fn name(&self) -> &str;
```
Returns the name of this plugin. Example:
```rust
impl Plugin for MyPlugin {
  fn name(&self) -> &str {
    "MyPlugin"
  }
}
```

## priority
- **required: `false`**
- **default:**
```rust
fn priority(&self) -> i32 {
  100
}
```

Define the priority of this plugin, the larger the value, the earlier this plugin execute. When plugins has same priority, they will be executed as the same order as the registered order in `plugins`.

:::note
By default, all custom plugin's priority is 100. And some internal plugins' order is 99, like `plugin-script`, `plugin-css`, you can override the internal plugin's behavior when default priority. But some internal plugins' priority is 101, like `plugin-resolve`, `plugin-html`, you should setup a larger priority if you want override the default behavior.
:::

## config
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn config(&self, _config: &mut Config) -> Result<Option<()>> {
  Ok(None)
}
```
Modify the config before compilation start in `config` hook. Refer to [Config](https://docs.rs/farmfe_core/latest/farmfe_core/config/struct.Config.html) for definition of Config struct. Example:

```rust
impl Plugin for MyPlugin {
  // implement config hook
  fn config(&self, config: &mut Config) -> Result<Option<()>> {
    // set minify to false 
    config.input.insert("custom-entry", "./custom.html");
    Ok(Some(()))
  }
}
```
Note that the `Rust Plugin`'s `config` hook are called after `JS Plugin`'s `config` and `configResolved` hook. 

## plugin_cache_loaded
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn plugin_cache_loaded(
  &self,
  _cache: &Vec<u8>,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```
Extend [`persistent cache`](/docs/advanced/persistent-cache) loading for your plugin.

When `Persistent Cache` enabled, `load` and `transform` hook may be skipped when hitting cache. If your plugin relies on previous compilation result(for example, load a virtual module based on existing modules), you may need to implement this hook to load cached infos of your plugin to ensure cache work as expected.

Example:
```rust
#[cache_item]
struct CachedStaticAssets {
  list: Vec<Resource>,
}

impl Plugin for StaticAssetsPlugin {
  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cached_static_assets: CachedAssets = deserialize!(cache, CachedStaticAssets);

    for asset in cached_static_assets.list {
      if let ResourceOrigin::Module(m) = asset.origin {
        context.emit_file(EmitFileParams {
          resolved_path: m.to_string(),
          name: asset.name,
          content: asset.bytes,
          resource_type: asset.resource_type,
        });
      }
    }

    Ok(Some(()))
  }
}
```
Note:
* `deserialize` is exposed by `farmfe_core`, it can help you deserialize your structs or enums from `Vec<u8>`.
* The cached structs or enums **must be rkyv serializable**, you can use `#[cache_item]` exposed by `farmfe_core` create a cacheable struct quickly.

## build_start
- **required: `false`**
- **hook type: `parallel`**
- **default:**
```rust
fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```
Called before the first compilation starts. You can use this hook to initialize any initial status of your plugins.

:::note
`build_start` is only called once for the first compilation. If you want to do something when ModuleGraph is updated in `HMR` or `Lazy Compilation`, you should use [update_modules](#update_modules) hook.
:::

## resolve
- **required: `false`**
- **hook type: `first`**
- **default:**
```rust
fn resolve(
  &self,
  _param: &PluginResolveHookParam,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<PluginResolveHookResult>> {
  Ok(None)
}

/// Parameter of the resolve hook
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginResolveHookParam {
  /// the source would like to resolve, for example, './index'
  pub source: String,
  /// the start location to resolve `specifier`, being [None] if resolving a entry or resolving a hmr update.
  pub importer: Option<ModuleId>,
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  pub kind: ResolveKind,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginResolveHookResult {
  /// resolved path, normally a absolute file path.
  pub resolved_path: String,
  /// whether this module should be external, if true, the module won't present in the final result
  pub external: bool,
  /// whether this module has side effects, affects tree shaking
  pub side_effects: bool,
  /// the query parsed from specifier, for example, query should be `{ inline: "" }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [farmfe_toolkit::resolve::parse_query] should be helpful
  pub query: Vec<(String, String)>,
  /// the meta data passed between plugins and hooks
  pub meta: HashMap<String, String>,
}
```

Custom `source` resolving from `importer`, for example, resolving `./b` from `a.ts`:
```ts title="a.ts"
import b from './b?raw';
// ...
```
Then the resolve params would be:
```rust
let param = PluginResolveHookParam {
  source: "./b",
  importer: Some(ModuleId { relative_path: "a.ts", query_string: "" }),
  kind: ResolveKind::Import
}
```
The resolve result of default resolver would be:
```rust
let resolve_result = PluginResolveHookResult {
  resolved_path: "/root/b.ts",   // resolved absolute path of the module
  external: false, // this module should be included in the final compiled resources and should not be external
  side_effects: false, // this module may be tree shaken as it does not contains side effects
  query: vec![("raw", "")], // query from the source.
  meta: HashMap::new()
}
```

The `HookContext` is used to pass status when you can the hooks recursively, for example, your plugin call `context.plugin_driver.resolve` in `resolve hook`:
```rust
impl Plugin for MyPlugin {
  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    // add a guard to avoid infinite loop
    if let Some(caller) = &hook_context.caller {
      if caller.as_str() == "FarmPluginCss" {
        return Ok(None);
      }
    }

    if matches!(param.kind, ResolveKind::CssAtImport | ResolveKind::CssUrl) {
      // if dep starts with '~', means it's from node_modules.
      // otherwise it's always relative
      let source = if let Some(striped_source) = param.source.strip_suffix('~') {
        striped_source.to_string()
      } else if !param.source.starts_with('.') {
        format!("./{}", param.source)
      } else {
        param.source.clone()
      };

      // call resolve recursively
      return context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source,
          ..param.clone()
        },
        context,
        &PluginHookContext {
          // pass caller we call resolve recursively
          caller: Some("FarmPluginCss".to_string()),
          meta: Default::default(),
        },
      );
    }

    Ok(None)
  }
}
```

In above example, we call `context.plugin_driver.resolve` and pass `caller` as parameter, then we should add a guard like `if caller.as_str() == "FarmPluginCss"` to avoid infinite loop.

Note:
* By default, you `resolve hook` are executed **after** the default resolver inside Farm, only the sources that can not be resolved by internal resolver will be passed to your plugin, which means if you want to override the default resolve, you need to set your **plugin's priority larger** than `101`.
* Usually `resolved_path` is the real absolute path that points to a file. But you can still return a `virtual module id` like `virtual:my-module`, but for virtual module you need to implement `load` hook to custom how to load your virtual module. And in Farm, `resolved_path + query = module_id`.
* `ResolveKind` presents the `import type`, Example values: `ResolveKind::Require`(imported by commonjs require), `ResolveKind::CssImport`(imported by css's import statement), etc.
* `meta` can be shared between plugins and hooks, you can get `meta` from params of `load`, `transform` and `parse` hooks in any plugin.

## load
- **required: `false`**
- **hook type: `first`**
- **default:**
```rust
fn load(
  &self,
  _param: &PluginLoadHookParam,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<PluginLoadHookResult>> {
  Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadHookParam<'a> {
  /// the module id string
  pub module_id: String,
  /// the resolved path from resolve hook
  pub resolved_path: &'a str,
  /// the query map
  pub query: Vec<(String, String)>,
  /// the meta data passed between plugins and hooks
  pub meta: HashMap<String, String>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadHookResult {
  /// the source content of the module
  pub content: String,
  /// the type of the module, for example [ModuleType::Js] stands for a normal javascript file,
  /// usually end with `.js` extension
  pub module_type: ModuleType,
  /// source map of the module
  pub source_map: Option<String>,
}
```

Custom how to load your module from a resolved module path or module id. For example, load a virtual module:
```rust
impl Plugin for MyPlugin {
  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    // only handle the specified path
    if param.resolved_path == "virtual:my-plugin" {
      return Ok(Some(
        PluginLoadHookResult {
          content: "import real from './real-path';",
          module_type: ModuleType::Js
          source_map: None,
        }
      ))
    }

    Ok(None)
  }
}
```

`module_type` and `content` is required when loading modules in your `load` hook. `source_map` is optional, you can return source map if you do transform in the `load` hook(which is not recommended, we recommend to use `transform` hook for this situation) or you load original source map from other locations.

## transform
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn transform(
  &self,
  _param: &PluginTransformHookParam,
  _context: &Arc<CompilationContext>,
) -> Result<Option<PluginTransformHookResult>> {
  Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginTransformHookParam<'a> {
  /// the module id string
  pub module_id: String,
  /// source content after load or transformed result of previous plugin
  pub content: String,
  /// module type after load
  pub module_type: ModuleType,
  /// resolved path from resolve hook
  pub resolved_path: &'a str,
  /// query from resolve hook
  pub query: Vec<(String, String)>,
  /// the meta data passed between plugins and hooks
  pub meta: HashMap<String, String>,
  /// source map chain of previous plugins
  pub source_map_chain: Vec<Arc<String>>,
}


#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  pub content: String,
  /// you can change the module type after transform.
  pub module_type: Option<ModuleType>,
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  pub source_map: Option<String>,
  /// if true, the previous source map chain will be ignored, and the source map chain will be reset to [source_map] returned by this plugin.
  pub ignore_previous_source_map: bool,
}
```

Do transformation based on **`module content`** and **`module type`**. Example for transforming `sass` to `css`:

```rust
impl Plugin for MyPlugin {
  // ignore other code ...
  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    // module type guard is neccessary
    if param.module_type == ModuleType::Custom(String::from("sass")) {
      // parse options
      const options = parse_options(&self.options, param.module_id);
      // compile sass to css
      let compile_result = compileSass(&param.content, options);

      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: compile_result.css,
        source_map: compile_result.source_map,
        // tell farm compiler that we have transformed this module to css
        module_type: Some(farmfe_core::module::ModuleType::Css),
        ignore_previous_source_map: false,
      }));
    }

    Ok(None)
  }
}
```
Normal steps for writing `transform hook`:
1. add a `if` guard based `module_type` or `resolved_path` or `module_id`
2. do transformation of the `content`
3. return the transformed `content`, `source_map` and `module_type`

For `ignore_previous_source_map`, if you handled `param.source_map_chain` and collapsed the source maps of previous plugins in the `transform hook`. You should set ignore_previous_source_map to `true` to ensure source map is correct. Otherwise, you should always set this option to `false` and leave source map chain handled by Farm.

:::note
`transform` hook is **content to content**. There is a similar hook called `process_module`, `process_module` is **ast to ast**. So if you want to transform the loaded content string, you need to use `transform` hook, and if you want to transform the `ast`, you should use `process_module` hook.
:::

## parse
- **required: `false`**
- **hook type: `first`**
- **default:**
```rust
fn parse(
  &self,
  _param: &PluginParseHookParam,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<ModuleMetaData>> {
  Ok(None)
}

#[derive(Debug)]
pub struct PluginParseHookParam {
  /// module id
  pub module_id: ModuleId,
  /// resolved path
  pub resolved_path: String,
  /// resolved query
  pub query: Vec<(String, String)>,
  pub module_type: ModuleType,
  /// source content(after transform)
  pub content: Arc<String>,
}


/// Module meta data shared by core plugins through the compilation
/// Meta data which is not shared by core plugins should be stored in [ModuleMetaData::Custom]
#[cache_item]
pub enum ModuleMetaData {
  Script(ScriptModuleMetaData),
  Css(CssModuleMetaData),
  Html(HtmlModuleMetaData),
  Custom(Box<dyn SerializeCustomModuleMetaData>),
}
```

Parse the **`transformed module content`** to `ast`. `Js/Jsx/Ts/Tsx`, `css` and `html` are supported natively by Farm. Normally you do not need to implement this hook unless you want to support a new `module_type` other than `Js/Jsx/Ts/Tsx`, `css` and `html`, use `ModuleMetaData::Custom` for this scenario.


## process_module
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn process_module(
  &self,
  _param: &mut PluginProcessModuleHookParam,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}

pub struct PluginProcessModuleHookParam<'a> {
  pub module_id: &'a ModuleId,
  pub module_type: &'a ModuleType,
  pub content: Arc<String>,
  pub meta: &'a mut ModuleMetaData,
}
```

Do transformation of the `parsed result`, usually do **`ast transformation`**. For example, Farm strip typescript in `process_module` hook:
```rust
impl Plugin for MyPlugin {
   fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !param.module_type.is_script() {
      return Ok(None);
    }
    // strip typescript
    if param.module_type.is_typescript() {
      swc_script_transforms::strip_typescript(param, &cm, context)?;
    }
    // ...ignore other code

    Ok(Some(()))
  }

}
```
In above example, we ignore non-script modules and strip type annotation of the ast for ts/tsx modules.

## analyze_deps
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn analyze_deps(
  &self,
  _param: &mut PluginAnalyzeDepsHookParam,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}

pub struct PluginAnalyzeDepsHookParam<'a> {
  pub module: &'a Module,
  /// analyzed deps from previous plugins, you can push new entries to it for your plugin.
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}
```

Analyze dependencies of the module. For example, we have `a.ts`:

```ts title="a.ts"
import b from './b';
const c = require('./c');
```

then normally this hook should push **2 entries** to `params.deps`:

```rust
param.deps.push(PluginAnalyzeDepsHookResultEntry {
  source: "./b".to_string(),
  kind: ResolveKind::Import
});
param.deps.push(PluginAnalyzeDepsHookResultEntry {
  source: "./c".to_string(),
  kind: ResolveKind::Require
});
```

`param.deps` will be passed to `resolve` hook later. You can also add new deps that is not related to the ast of your module as you wish, Farm will `resolve`, `load` these unrelated modules and add them to the module graph too.

## finalize_modules
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn finalize_module(
  &self,
  _param: &mut PluginFinalizeModuleHookParam,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}

pub struct PluginFinalizeModuleHookParam<'a> {
  pub module: &'a mut Module,
  pub deps: &'a Vec<PluginAnalyzeDepsHookResultEntry>,
}
```

Do any thing you want before seal the module. Note that you can only modify `param.module`.

## build_end
- **required: `false`**
- **hook type: `parallel`**
- **default:**
```rust
/// The module graph should be constructed and finalized here
fn build_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```
Called when all dependencies starting from `config.input` are handled and `ModuleGraph` is successfully constructed, you can get the full resolved `ModuleGraph` here by `context.module_graph`.

:::note
`build_end` is only called once for the first compilation. If you want to do something when ModuleGraph is updated in `HMR` or `Lazy Compilation`, you should use [module_graph_updated](#module_graph_updated) hook.
:::

## generate_start
- **required: `false`**
- **hook type: `parallel`**
- **default:**
```rust
fn generate_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```

Called before generate stage start.

## optimize_module_graph
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
/// Some optimization of the module graph should be performed here, for example, tree shaking
fn optimize_module_graph(
  &self,
  _module_graph: &mut ModuleGraph,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```
You can do optimization of the `module_graph` here. For internal plugins, Farm does tree shaking, minification in this hook.

## analyze_module_graph
- **required: `false`**
- **hook type: `first`**
- **default:**
```rust
/// Analyze module group based on module graph
fn analyze_module_graph(
  &self,
  _module_graph: &mut ModuleGraph,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<ModuleGroupGraph>> {
  Ok(None)
}
```
Analyze **`dynamic import`** of the `module_graph`, and groups modules based on **`dynamic import`**, return the grouped modules.

:::warning
Normally you should not implement this hook, unless you want to implement a full new bundling algorithm in Farm.
:::

## partial_bundling
- **required: `false`**
- **hook type: `first`**
- **default:**
```rust
/// partial bundling modules to [Vec<ResourcePot>]
fn partial_bundling(
  &self,
  _modules: &Vec<ModuleId>,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<Vec<ResourcePot>>> {
  Ok(None)
}

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePot {
  pub id: ResourcePotId,
  pub name: String,
  pub resource_pot_type: ResourcePotType,
  modules: HashSet<ModuleId>,
  pub meta: ResourcePotMetaData,
  /// [None] if this [ResourcePot] does not contain entry module.
  /// [Some(entry_id)] otherwise
  pub entry_module: Option<ModuleId>,
  /// the resources generated in this [ResourcePot]
  resources: HashSet<String>,

  /// This field should be filled in partial_bundling_hooks.
  /// the module groups that this [ResourcePot] belongs to.
  /// A [ResourcePot] can belong to multiple module groups.
  pub module_groups: HashSet<ModuleGroupId>,
  pub immutable: bool,
}
```
Bundle the `modules` to `Vec<ResourcePot>` based on `module_group_graph` and `module_graph`. A `ResourcePot` is a structure that Farm uses to hold bundled modules, it will be emitted to final resources in [generate_resources](#generate_resources) hook, you can treat a `ResourcePot` as `Chunk` of other tools.

Note:
* This hook will be called in both `first compilation`, `HMR` and `Lazy Compilation`, make sure this hook does not contains side effects(for the same modules, always returns the same `Vec<ResourcePot>`).
* You should set `module.resource_pot` in this hook.

Refer to the [internal implementation](https://github.com/farm-fe/farm/blob/main/crates/plugin_partial_bundling/src/lib.rs) of partial bundling in Farm for best practice. Refer to [RFC-003 Partial Bundling](https://github.com/farm-fe/farm/blob/main/crates/plugin_partial_bundling/src/lib.rs) for how Farm designs bundling.

:::warning
Normally you should not implement this hook, unless you want to implement a full new bundling algorithm in Farm. And If you override this hook, `config.partial_bundling` may not work unless you follow the same bundling spec as Farm.
:::

## process_resource_pots
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
/// process resource pots before render and generating each resource
fn process_resource_pots(
  &self,
  _resource_pots: &mut Vec<&mut ResourcePot>,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```
Do some **transformation** of the `ResourcePots`. Note that ResourcePots are not rendered at this time, which means you can not get the rendered code of the `Resource Pot`, instead, you can only add, remove, transform the modules inside the `ResourcePot` 

## render_start
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn render_start(
  &self,
  _config: &Config,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```

Called before resource pots render. After rendering resource pots, executable `html`, `css`, `js`, etc files will be emitted.

:::note
`render_start` is only called once for the first compilation. `HMR` or `Lazy Compilation` won't trigger `render_start` hook.
:::

## render_resource_pot_modules
- **required: `false`**
- **hook type: `first`**
- **default:**
```rust
fn render_resource_pot_modules(
  &self,
  _resource_pot: &ResourcePot,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<ResourcePotMetaData>> {
  Ok(None)
}

#[cache_item]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RenderedModule {
  pub id: ModuleId,
  pub rendered_content: Arc<String>,
  pub rendered_map: Option<Arc<String>>,
  pub rendered_length: usize,
  pub original_length: usize,
}

#[cache_item]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePotMetaData {
  pub rendered_modules: HashMap<ModuleId, RenderedModule>,
  pub rendered_content: Arc<String>,
  pub rendered_map_chain: Vec<Arc<String>>,
  pub custom_data: HashMap<String, String>,
}
```

Render the given `ResourcePot` to `rendered_content` and `rendered_source_map_chain`. This hook is used to render `module's ast` to bundled code. If you just want to modify the bundled code, use [render_resource_pot](#render_resource_pot) instead.

If you really need to use this hook, refer to [plugin_runtime](https://github.com/farm-fe/farm/blob/main/crates/plugin_runtime/src/lib.rs#L320) for best practice.

:::note
Normally you should not override this hook for natively supported module types like `js/jsx/ts/tsx/css/html`, you should only use this hook when you ensure you want to override the default behavior for internal module types in Farm, or you want to support **custom module types**. 
:::

## render_resource_pot
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn render_resource_pot(
  &self,
  _param: &PluginRenderResourcePotHookParam,
  _context: &Arc<CompilationContext>,
) -> Result<Option<PluginRenderResourcePotHookResult>> {
  Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRenderResourcePotHookParam {
  pub content: Arc<String>,
  pub source_map_chain: Vec<Arc<String>>,
  pub resource_pot_info: ResourcePotInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginRenderResourcePotHookResult {
  pub content: String,
  pub source_map: Option<String>,
}
```

Transform the rendered bundled code for the given `ResourcePot`. Return `rendered content` and `source map`.

```rust
impl Plugin for MyPlugin {
  fn render_resource_pot(
    &self,
    param: &PluginRenderResourcePotHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<PluginRenderResourcePotHookResult>> {
    if (param.resource_pot_info.resource_pot_type == ResourcePotType::Css) {
      return Ok(Some(PluginRenderResourcePotHookResult {
        content: param.content.replaceAll("<--layer-->", replaced_code),
        source_map: replaced_map,
      }))
    }

    Ok(None)
  }
}
```
In above example, we transformed the content of a css `Resource Pot`, replaced all `<--layer-->` to `replaced_code`.

## augment_resource_hash
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn augment_resource_hash(
  &self,
  _render_pot_info: &ResourcePotInfo,
  _context: &Arc<CompilationContext>,
) -> Result<Option<String>> {
  Ok(None)
}

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePotInfo {
  pub id: ResourcePotId,
  pub name: String,
  pub resource_pot_type: ResourcePotType,
  pub module_ids: Vec<ModuleId>,
  pub map: Option<Arc<String>>,
  pub modules: HashMap<ModuleId, RenderedModule>,
  pub data: ResourcePotInfoData,
}
```

Append additional hash when generating resource from given resource pot.

## optimize_resource_pot
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
/// Optimize the resource pot, for example, minimize
fn optimize_resource_pot(
  &self,
  _resource_pot: &mut ResourcePot,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}

#[cache_item]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePot {
  pub id: ResourcePotId,
  pub name: String,
  pub resource_pot_type: ResourcePotType,
  modules: HashSet<ModuleId>,
  pub meta: ResourcePotMetaData,
  /// [None] if this [ResourcePot] does not contain entry module.
  /// [Some(entry_id)] otherwise
  pub entry_module: Option<ModuleId>,
  /// the resources generated in this [ResourcePot]
  resources: HashSet<String>,

  /// This field should be filled in partial_bundling_hooks.
  /// the module groups that this [ResourcePot] belongs to.
  /// A [ResourcePot] can belong to multiple module groups.
  pub module_groups: HashSet<ModuleGroupId>,
  pub immutable: bool,
  pub info: Box<ResourcePotInfo>,
}
```

Do some optimizations for the rendered resource pot. For example, minification. If you want to modify the rendered content of this hook, just modify `resource_pot.meta.rendered_content` and append sourcemap of this transformation in `resource_pot.meta.rendered_source_map_chain`.

:::note
Optimizations like minification is handled internally by Farm, make sure that you really need to use this hook.
:::

## generate_resources
- **required: `false`**
- **hook type: `first`**
- **default:**
```rust
/// Generate resources based on the [ResourcePot], return [Vec<Resource>] represents the final generated files.
/// For example, a .js file and its corresponding source map file
fn generate_resources(
  &self,
  _resource_pot: &mut ResourcePot,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<PluginGenerateResourcesHookResult>> {
  Ok(None)
}

#[cache_item]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginGenerateResourcesHookResult {
  pub resource: Resource,
  pub source_map: Option<Resource>,
}
```

Generate final resource for the give rendered resource pot. return `generated resource` and `optional source map resource`.

:::note
For natively supported `ModuleTypes` like `js/ts/jsx/tsx/css/html/static assets`, normally you do not need to implement this hook. Use this hook when you want to support a new type of resource that not natively supported by Farm.
:::

## finalize_resources
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
/// Do some finalization work on the generated resources, for example, transform html based on the generated resources
fn finalize_resources(
  &self,
  _param: &mut PluginFinalizeResourcesHookParams,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}

pub struct PluginFinalizeResourcesHookParams<'a> {
  pub resources_map: &'a mut HashMap<String, Resource>,
  pub config: &'a Config,
}
```

Do some finalization work on the generated resources, for example, transform html based on the generated resources(insert `<script>`, `<link>` tags).

You can also **`add`** or **`remove`** resources here.

## generate_end
- **required: `false`**
- **hook type: `parallel`**
- **default:**
```rust
fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```

Called when all generate stage done(including `finalize_resources`). You can do some cleanup work here.

## finish
- **required: `false`**
- **hook type: `parallel`**
- **default:**
```rust
fn finish(&self, _stat: &Stats, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```

Called when all compilation work done(including `build stage` and `generate stage`). You can do some cleanup work here.

:::note
`finish` is only called once for the first compilation. `HMR` or `Lazy Compilation` won't trigger `finish` hook. You should use [update_finished](#update_finished) hook instead.
:::


## write_plugin_cache
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn write_plugin_cache(&self, _context: &Arc<CompilationContext>) -> Result<Option<Vec<u8>>> {
  Ok(None)
}
```

Extend [`persistent cache`](/docs/advanced/persistent-cache) writing for your plugin. `write_plugin_cache` is often used together with [plugin_cache_loaded](#plugin_cache_loaded) to read and write persistent cache for plugin. Return the serialized bytes of your data.

Example, writing cache for static assets:
```rust
impl Plugin for MyPlugin {
  fn write_plugin_cache(
    &self,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    let mut list = vec![];
    let resources_map = context.resources_map.lock();

    for (_, resource) in resources_map.iter() {
      if let ResourceOrigin::Module(m) = &resource.origin {
        if context.module_graph.read().has_module(m) {
          list.push(resource.clone());
        }
      }
    }

    if !list.is_empty() {
      let cached_static_assets = CachedStaticAssets { list };

      Ok(Some(serialize!(&cached_static_assets)))
    } else {
      Ok(None)
    }
  }
}

#[cache_item]
struct CachedStaticAssets {
  list: Vec<Resource>,
}
```


## update_modules
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
/// Called when calling compiler.update(module_paths).
/// Useful to do some operations like clearing previous state or ignore some files when performing HMR
fn update_modules(
  &self,
  _params: &mut PluginUpdateModulesHookParams,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginUpdateModulesHookParams {
  pub paths: Vec<(String, UpdateType)>,
}
```
Called when calling compiler.update(module_paths). Useful to do some operations like clearing previous state or ignore some files when performing HMR

* `paths` is paths that will be recompiled for this update
* return the new `paths`, later compilation will update the new returned paths.

## module_graph_updated
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
/// Called when calling compiler.update(module_paths).
/// Useful to do some operations like modifying the module graph
fn module_graph_updated(
  &self,
  _param: &PluginModuleGraphUpdatedHookParams,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginModuleGraphUpdatedHookParams {
  pub added_modules_ids: Vec<ModuleId>,
  pub removed_modules_ids: Vec<ModuleId>,
  pub updated_modules_ids: Vec<ModuleId>,
}
```
Called when calling compiler.update(module_paths). Useful to do some operations like modifying the module graph.

## update_finished
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
/// Called when calling compiler.update(module_paths).
/// This hook is called after all compilation work is done, including the resources regeneration and finalization.
fn update_finished(
  &self,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```

Called when calling compiler.update(module_paths). This hook is called after all compilation work is done, including the resources regeneration and finalization.

## handle_persistent_cached_module
- **required: `false`**
- **hook type: `serial`**
- **default:**
```rust
fn handle_persistent_cached_module(
  &self,
  _module: &farmfe_core::module::Module,
  _context: &Arc<CompilationContext>,
) -> Result<Option<bool>> {
  Ok(None)
}
```
Called when persistent cache is enabled and the cache hit for the module. Return `true` to **skip loading cache for this module**.