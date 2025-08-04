# Rust Plugin Api

:::note
本文档仅涵盖插件钩子详情。 有关如何创建、构建和发布 rust 插件，请参阅：[编写 Rust 插件](/docs/plugins/writing-plugins/rust-plugin)
:::

## 配置 Rust 插件

通过 `plugins` 选项添加 Rust 插件：

```ts title="farm.config.ts" {3,7}
import { defineConfig } from "farm";

export default defineConfig({
  // 在 plugins 中配置
  plugins: [
    [
      "@farmfe/plugin-sass",
      {
        /** 插件选项 */
      },
    ],
  ],
});
```

在字符串中配置 Rust 插件包名称（或路径），并在对象中配置其选项。

## 编写 Rust 插件

有关详细信息，请参阅[编写 Rust 插件](/docs/plugins/writing-plugins/rust-plugin)。

## 插件 Hook 概述

Farm 提供了很多 rollup 风格的 hook，这些 hook 分为 build 阶段和 generate 阶段：
![Farm 插件 钩子](/img/farm-plugin-hooks.png)

所有插件挂钩都接受一个名为 [`CompilationContext`](https://docs.rs/farmfe_core/latest/farmfe_core/context/struct.CompilationContext.html) 的参数。 所有共享的编译信息都存储在 `context` 中。

Hook 共有三种（与 Rollup 相同）：

- `first`：钩子串行执行，当钩子返回非空值时立即返回。 （null 在 JS 中表示 null 和 undefined，在 Rust 中表示 None）。
- `serial`: 钩子串行执行，每个钩子的结果将传递到下一个钩子，使用最后一个钩子的结果作为最终结果。
- `parallel`：钩子在线程池中并行执行，并且应该被隔离。

:::note
有关完整的 `Plugin Hooks` 定义，请参阅[Plugin Trait](https://docs.rs/farmfe_core/latest/farmfe_core/plugin/trait.Plugin.html)
:::

## name

- **required: `true`**
- **default:**

```rust
fn name(&self) -> &str;
```

返回此插件的名称。 例子：

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

定义该插件的优先级，值越大，该插件越早执行。 当插件具有相同优先级时，它们将按照与`plugins`中注册的顺序相同的顺序执行。

:::note
默认情况下，所有自定义插件的优先级都是 100。有些内部插件的优先级是 99，比如 `plugin-script`、`plugin-css`，您可以覆盖默认优先级时内部插件的行为。 但是一些内部插件的优先级是101，比如`plugin-resolve`，`plugin-html`，如果你想覆盖默认行为，你应该设置一个更大的优先级。
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

在编译开始之前在`config`钩子中修改配置。 Config 结构体的定义请参考[Config](https://docs.rs/farmfe_core/latest/farmfe_core/config/struct.Config.html)。 例子：

```rust
impl Plugin for MyPlugin {
  // 实现 config hook
  fn config(&self, config: &mut Config) -> Result<Option<()>> {
    // 设置 minify 为 false
    config.input.insert("custom-entry", "./custom.html");
    Ok(Some(()))
  }
}
```

请注意， `Rust Plugin` 的 `config` 钩子是在 `JS Plugin` 的 `config` 和 `configResolved` 钩子之后调用的。

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

扩展插件的持久缓存加载。

当启用 `持久缓存` 时，在命中缓存时可能会跳过 `加载` 和 `转换` 挂钩。 如果您的插件依赖于以前的编译结果（例如，基于现有模块加载虚拟模块），您可能需要实现此钩子来加载插件的缓存信息，以确保缓存按预期工作。

例子：

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

注意：

- `deserialize` 由 `farmfe_core` 导出，它可以帮助您反序列化 `Vec<u8>` 中的结构体或枚举。
- 缓存的结构体或枚举**必须是rkyv可序列化的**，您可以使用`farmfe_core`公开的`#[cache_item]`快速创建可缓存的结构体。

## build_start

- **required: `false`**
- **hook type: `parallel`**
- **default:**

```rust
fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```

在第一次编译开始之前调用。 您可以使用此挂钩来初始化插件的任何初始状态。

:::note
`build_start` 仅在第一次编译时调用一次。 如果你想在`HMR`或`Lazy Compilation`中更新ModuleGraph时做一些事情，你应该使用[update_modules](#update_modules)钩子。
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

/// 解析钩子的参数
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PluginResolveHookParam {
  /// 我们想要解析的源，例如'./index'
  pub source: String,
  /// 解析 `specifier` 的起始位置，如果解析入口或解析 hmr 更新，则为 [None]。
  pub importer: Option<ModuleId>,
  /// 例如，[ResolveKind::Import] 用于静态导入 (`import a from './a'`)
  pub kind: ResolveKind,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginResolveHookResult {
  /// 解析路径，通常是绝对文件路径。
  pub resolved_path: String,
  /// 该模块是否应该被 external，如果为 true，则该模块不会出现在最终结果中
  pub external: bool,
  /// 是否具有副作用，影响 tree shake
  pub side_effects: bool,
  /// 从说明符解析的查询，例如，如果说明符是`./a.png?inline`，查询应该是`{ inline: "" }`
  /// 如果你自定义插件，你的插件应该负责解析查询
  /// 如果您只想像上面的示例一样进行正常的查询解析， [farmfe_toolkit::resolve::parse_query] 应该会有所帮助
  pub query: Vec<(String, String)>,
  /// 插件和钩子之间传递的元数据
  pub meta: HashMap<String, String>,
}
```

从 `importer` 解析自定义 `source` ，例如从 `a.ts` 解析 `./b` ：

```ts title="a.ts"
import b from "./b?raw";
// ...
```

那么解析参数将是：

```rust
let param = PluginResolveHookParam {
  source: "./b",
  importer: Some(ModuleId { relative_path: "a.ts", query_string: "" }),
  kind: ResolveKind::Import
}
```

默认解析器的解析结果为：

```rust
let resolve_result = PluginResolveHookResult {
  resolved_path: "/root/b.ts",   // 解析模块的绝对路径
  external: false, // 该模块应该包含在最终编译的资源中，并且不应该是外部的
  side_effects: false, // 无副作用
  query: vec![("raw", "")], // query
  meta: HashMap::new()
}
```

`HookContext` 用于在您可以递归挂钩时传递状态，例如，您的插件在 `resolve hook` 中调用 `context.plugin_driver.resolve`：

```rust
impl Plugin for MyPlugin {
  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    // 添加一个守卫以避免无限循环
    if let Some(caller) = &hook_context.caller {
      if caller.as_str() == "FarmPluginCss" {
        return Ok(None);
      }
    }

    if matches!(param.kind, ResolveKind::CssAtImport | ResolveKind::CssUrl) {
      // 如果dep以'~'开头，则表示它来自node_modules。
      // 否则它总是相对的
      let source = if let Some(striped_source) = param.source.strip_suffix('~') {
        striped_source.to_string()
      } else if !param.source.starts_with('.') {
        format!("./{}", param.source)
      } else {
        param.source.clone()
      };

      // 递归调用resolve
      return context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source,
          ..param.clone()
        },
        context,
        &PluginHookContext {
          caller: Some("FarmPluginCss".to_string()),
          meta: Default::default(),
        },
      );
    }

    Ok(None)
  }
}
```

在上面的示例中，我们调用 `context.plugin_driver.resolve` 并将 `caller` 作为参数传递，然后我们应该添加一个类似 `if caller.as_str() == "FarmPluginCss"` 的保护以避免无限循环。

注意：

- 默认情况下，您的`resolve hook`在Farm内部默认解析器**之后**执行，只有内部解析器无法解析的源才会传递给您的插件，这意味着如果您想覆盖默认解析器 ，您需要将**插件的优先级设置为大于**`101`。
- 通常 `resolved_path` 是指向文件的真实绝对路径。 但是您仍然可以返回一个 `虚拟模块 id` ，例如 `virtual:my-module` ，但是对于虚拟模块，您需要实现 `load` 钩子来自定义如何加载虚拟模块。 在 Farm 中，`resolved_path + query = module_id`。
- `ResolveKind` 表示 `导入类型`，示例值：`ResolveKind::Require`（由 commonjs require 导入）、`ResolveKind::CssImport`（由 css 的 import 语句导入）等。
- `meta` 可以在插件和钩子之间共享，您可以从任何插件中的 `load`、`transform` 和 `parse` 钩子的参数中获取 `meta`。

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
  /// 模块id字符串
  pub module_id: String,
  /// 来自解析钩子的解析路径
  pub resolved_path: &'a str,
  pub query: Vec<(String, String)>,
/// 插件和钩子之间传递的元数据
  pub meta: HashMap<String, String>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginLoadHookResult {
  /// 模块的源内容
  pub content: String,
  /// 模块的类型，例如[ModuleType::Js]代表普通的javascript文件，
  /// 通常以 `.js` 扩展名结尾
  pub module_type: ModuleType,
  pub source_map: Option<String>,
}
```

自定义如何从已解析的模块路径或模块 ID 加载模块。 例如加载一个虚拟模块：

```rust
impl Plugin for MyPlugin {
  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    // 只处理指定路径
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

在 `load` 钩子中加载模块时需要 `module_type` 和 `content` 。 `source_map` 是可选的，如果您在 `load` 钩子中进行转换（不推荐，我们建议在这种情况下使用 `transform` 钩子）或者从其他位置加载原始源地图，则可以返回源地图。

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
  /// 模块id字符串
  pub module_id: String,
  /// 加载后的源内容或上一个插件转换后的结果
  pub content: String,
  /// 加载后的模块类型或者上一个插件转换后的类型
  pub module_type: ModuleType,
  /// resolve 的绝对路径
  pub resolved_path: &'a str,
  pub query: Vec<(String, String)>,
  pub meta: HashMap<String, String>,
  /// 之前插件的 source map 链
  pub source_map_chain: Vec<Arc<String>>,
}


#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PluginTransformHookResult {
  /// 转换后的源内容，将传递给下一个插件。
  pub content: String,
  /// 您可以在转换后更改模块类型。
  pub module_type: Option<ModuleType>,
  /// 转换后的源映射，所有插件转换后的源映射将存储为源映射链。
  pub source_map: Option<String>,
  /// 如果为true，则之前的源映射链将被忽略，并且源映射链将重置为该插件返回的[source_map]。
  pub ignore_previous_source_map: bool,
}
```

根据**`模块内容`**和**`模块类型`**进行转换。 将 `sass` 转换为 `css` 的示例：

```rust
impl Plugin for MyPlugin {
  // 忽略其他代码...
  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    // 模块类型判断是必要的
    if param.module_type == ModuleType::Custom(String::from("sass")) {
      // parse options
      const options = parse_options(&self.options, param.module_id);
      // compile sass to css
      let compile_result = compileSass(&param.content, options);

      return Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content: compile_result.css,
        source_map: compile_result.source_map,
        // 告诉 farm 编译器我们已经将此模块转换为 css
        module_type: Some(farmfe_core::module::ModuleType::Css),
        ignore_previous_source_map: false,
      }));
    }

    Ok(None)
  }
}
```

编写 `transform hook` 的正常步骤：

1. 添加基于 `module_type` 或 `resolved_path` 或 `module_id` 的 `if` 保护
2. 对 `内容` 进行转换3.返回转换后的`content`、`source_map`和`module_type`

对于 `ignore_previous_source_map` ，如果您处理了 `param.source_map_chain` 并折叠了 `transform hook` 中以前插件的 source map。 您应该将ignore_previous_source_map设置为 `true` 以确保 source map 正确。 否则，您应该始终将此选项设置为 `false` 并让 Farm 处理 source map 链。

:::note
`transform` 钩子是**内容到内容**。 有一个类似的钩子叫做 `process_module` ， `process_module` 是**ast to ast**。 所以如果你想转换加载的内容字符串，你需要使用`transform`钩子，如果你想转换`ast`，你应该使用`process_module`钩子。
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


/// 核心插件通过编译共享的模块元数据
/// 核心插件不共享的元数据应存储在 [ModuleMetaData::Custom] 中
#[cache_item]
pub enum ModuleMetaData {
  Script(ScriptModuleMetaData),
  Css(CssModuleMetaData),
  Html(HtmlModuleMetaData),
  Custom(Box<dyn SerializeCustomModuleMetaData>),
}
```

将**`转换后的模块内容`**解析为`ast`。 Farm 原生支持 `Js/Jsx/Ts/Tsx`、`css` 和 `html`。 通常你不需要实现这个钩子，除非你想支持除了`Js/Jsx/Ts/Tsx`、`css`和`html`之外的新的`module_type`，在这种情况下使用`ModuleMetaData::Custom`。

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

对`解析结果`进行变换，通常做**`ast变换`**。 例如，Farm 将 ts 转换成 js：

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
    // 去除 ts 类型
    if param.module_type.is_typescript() {
      swc_script_transforms::strip_typescript(param, &cm, context)?;
    }
    // ...ignore other code

    Ok(Some(()))
  }

}
```

在上面的示例中，我们忽略非脚本模块，并且去掉 ts/tsx 模块的 ast 中类型信息。

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
  /// 分析了以前插件的依赖关系，您可以为您的插件推送新条目。
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}
```

分析模块的依赖关系。 例如，我们有 `a.ts` ：

```ts title="a.ts"
import b from "./b";
const c = require("./c");
```

那么通常这个钩子应该将**2个条目**推送到`params.deps`：

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

`param.deps` 稍后将传递给 `resolve` 钩子。 您还可以根据需要添加与模块的 ast 不相关的新 deps，Farm 将 `resolve` 、 `load` 这些不相关的模块并将它们添加到模块图中。

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

在密封模块之前做任何你想做的事情。 请注意，您只能修改 `param.module` 。

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

当从`config.input`开始的所有依赖都被处理并且`ModuleGraph`被成功构建时调用，您可以通过`context.module_graph`在这里获得完整解析的`ModuleGraph`。

:::note
`build_end` 仅在第一次编译时调用一次。 如果你想在`HMR`或`Lazy Compilation`中更新ModuleGraph时做一些事情，你应该使用[module_graph_updated](#module_graph_updated)钩子。
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

在生成阶段开始之前调用。

## optimize_module_graph

- **required: `false`**
- **hook type: `serial`**
- **default:**

```rust
/// 这里应该对模块图进行一些优化，例如tree shake
fn optimize_module_graph(
  &self,
  _module_graph: &mut ModuleGraph,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```

您可以在此处对 `module_graph` 进行优化。 对于内部插件，Farm 在这个钩子中进行树摇动和缩小。

## analyze_module_graph

- **required: `false`**
- **hook type: `first`**
- **default:**

```rust
/// 根据模块图分析模块组
fn analyze_module_graph(
  &self,
  _module_graph: &mut ModuleGraph,
  _context: &Arc<CompilationContext>,
  _hook_context: &PluginHookContext,
) -> Result<Option<ModuleGroupGraph>> {
  Ok(None)
}
```

分析`module_graph`的**`动态导入`**，并根据**`动态导入`**对模块进行分组，返回分组后的模块。

:::warning
通常你不应该实现这个钩子，除非你想在 Farm 中实现一个全新的打包算法。
:::

## partial_bundling

- **required: `false`**
- **hook type: `first`**
- **default:**

```rust
/// 局部打包模块到 [Vec<ResourcePot>]
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
  /// 如果此 [ResourcePot] 不包含入口模块，则 [None]。
  /// [一些(entry_id)] 否则
  pub entry_module: Option<ModuleId>,
  /// 这个[ResourcePot]中生成的资源
  resources: HashSet<String>,

  /// 此 [ResourcePot] 所属的模块组。
  /// 一个[ResourcePot]可以属于多个模块组。
  pub module_groups: HashSet<ModuleGroupId>,
  pub immutable: bool,
}
```

基于 `module_group_graph` 和 `module_graph` 将 `modules` 打包到 `Vec<ResourcePot>` 。 `ResourcePot` 是 Farm 用于保存打包模块的结构，它将被生成到 [generate_resources](#generate_resources) 钩子中的最终资源，您可以将 `ResourcePot` 视为其他工具的 `Chunk`。

注意：

- 该钩子会在 `首次编译` 、 `HMR` 和 `延迟编译` 中被调用，请确保该钩子不包含副作用（对于相同的模块，始终返回相同的 `Vec<ResourcePot>` ）。
- 你应该在这个钩子中设置`module.resource_pot`。

请参阅 Farm 中部分捆绑的[内部实现](https://github.com/farm-fe/farm/blob/main/crates/plugin_partial_bundling/src/lib.rs)以获得最佳实践。 请参阅 [RFC-003 部分捆绑](https://github.com/farm-fe/farm/blob/main/crates/plugin_partial_bundling/src/lib.rs) 了解 Farm 如何设计捆绑。

:::warning
通常你不应该实现这个钩子，除非你想在 Farm 中实现一个全新的打包算法。 如果您覆盖此挂钩，除非您遵循与 Farm 相同的打包规范，否则 `config.partial_bundling` 可能无法工作。
:::

## process_resource_pots

- **required: `false`**
- **hook type: `serial`**
- **default:**

```rust
/// 在渲染和生成每个资源之前处理 resource pot
fn process_resource_pots(
  &self,
  _resource_pots: &mut Vec<&mut ResourcePot>,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```

对 `ResourcePots` 进行一些**转换**。 注意，此时ResourcePots还没有渲染，这意味着你无法获取`ResourcePot`的渲染代码，只能添加，删除，改造`ResourcePot`内部的模块

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

在 resource pot 渲染之前调用。 渲染 resource pot 后，将产出可执行的 `html` 、 `css` 、 `js` 等文件。

:::note
`render_start` 仅在第一次编译时调用一次。 `HMR` 或 `Lazy Compilation` 不会触发 `render_start` 钩子。
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

将给定的 `ResourcePot` 渲染为 `rendered_content` 和 `rendered_source_map_chain` 。 该钩子用于将 `模块的 ast` 渲染为打包代码。 如果您只想修改打包代码，请改用 [render_resource_pot](#render_resource_pot)。

如果您确实需要使用此钩子，请参阅 [plugin_runtime](https://github.com/farm-fe/farm/blob/main/crates/plugin_runtime/src/lib.rs#L320) 以获得最佳实践。

:::note
通常，您不应该为原生支持的模块类型（如 `js/jsx/ts/tsx/css/html` ）覆盖此钩子，只有当您确保要覆盖 Farm 中内部模块类型的默认行为时，才应该使用此钩子， 或者您想支持**自定义模块类型**。
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

转换给定 `ResourcePot` 的渲染捆绑代码。 返回 `渲染内容` 和 `source map` 。

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

在上面的示例中，我们转换了 css `Resource Pot` 的内容，将所有 `<--layer-->` 替换为 `replaced_code`。

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

从给定 resource pot 生成资源时附加额外哈希。

## optimize_resource_pot

- **required: `false`**
- **hook type: `serial`**
- **default:**

```rust
/// 优化资源罐，例如压缩
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

  pub entry_module: Option<ModuleId>,
  resources: HashSet<String>,

  pub module_groups: HashSet<ModuleGroupId>,
  pub immutable: bool,
  pub info: Box<ResourcePotInfo>,
}
```

对渲染的 resource pot 进行一些优化。 例如，替换、压缩等等。 如果要修改此钩子的渲染内容，只需修改 `resource_pot.meta.rendered_content` 并将此转换的 source map 附加到 `resource_pot.meta.rendered_source_map_chain` 中。

:::note
像压缩这样的优化是由 Farm 内部处理的，请确保您确实需要使用这个钩子。
:::

## generate_resources

- **required: `false`**
- **hook type: `first`**
- **default:**

```rust
/// 根据[ResourcePot]生成资源，返回[Vec<Resource>]代表最终生成的文件。
/// 例如一个.js文件及其对应的source map文件
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

为给定的渲染 resource pot 生成最终资源。 返回 `生成的资源` 和 `可选的 source map` 。

:::note
对于原生支持的 `ModuleTypes` ，如 `js/ts/jsx/tsx/css/html/static assets` ，通常不需要实现此钩子。 当您想要支持 Farm 本身不支持的新型资源时，请使用此钩子。
:::

## finalize_resources

- **required: `false`**
- **hook type: `serial`**
- **default:**

```rust
/// 对生成的资源做一些终结工作，例如根据生成的资源转换html
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

对生成的资源进行一些最终的处理工作，例如根据生成的资源转换html（插入`<script>`、`<link>`标签）。

您还可以在此处 **`添加`** 或 **`删除`** 资源。

## generate_end

- **required: `false`**
- **hook type: `parallel`**
- **default:**

```rust
fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```

当所有生成阶段完成时调用（包括 `finalize_resources` ）。 您可以在这里做一些清理工作。

## finish

- **required: `false`**
- **hook type: `parallel`**
- **default:**

```rust
fn finish(&self, _stat: &Stats, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
  Ok(None)
}
```

当所有编译工作完成时调用（包括 `构建阶段` 和 `生成阶段` ）。 您可以在这里做一些清理工作。

:::note
`finish` 仅在第一次编译时调用一次。 `HMR` 或 `Lazy Compilation` 不会触发 `finish` 钩子。 您应该使用 [update_finished](#update_finished) 钩子代替。
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

扩展插件的持久缓存写入。 `write_plugin_cache` 通常与 [plugin_cache_loaded](#plugin_cache_loaded) 一起使用来读写插件的持久缓存。 返回数据的序列化字节。

例如，为静态资源写入缓存：

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
/// 调用compiler.update(module_paths)时调用。
/// 在执行 HMR 时可用于执行一些操作，例如清除以前的状态或忽略某些文件
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

调用compiler.update(module_paths)时调用。 在执行 HMR 时可用于执行一些操作，例如清除以前的状态或忽略某些文件

- `paths` 是将为此更新重新编译的路径
- 返回新的`paths`，后续编译将更新返回的新路径。

## module_graph_updated

- **required: `false`**
- **hook type: `serial`**
- **default:**

```rust
/// 调用compiler.update(module_paths)时调用。
/// 用于执行一些操作，例如修改模块图
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

调用compiler.update(module_paths)时调用。 对于执行一些操作（例如修改模块图）很有用。

## update_finished

- **required: `false`**
- **hook type: `serial`**
- **default:**

```rust
/// 调用compiler.update(module_paths)时调用。
/// 该钩子在所有编译工作完成后调用，包括资源重新生成和终结。
fn update_finished(
  &self,
  _context: &Arc<CompilationContext>,
) -> Result<Option<()>> {
  Ok(None)
}
```

调用compiler.update(module_paths)时调用。 该钩子在所有编译工作完成后调用，包括资源重新生成和最终处理。

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

当启用持久缓存并且模块的缓存命中时调用。 返回 `true` 以**跳过加载此模块的缓存**。
