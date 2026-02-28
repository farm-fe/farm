# Farm v2 Codebase Analysis - Configuration Types

## Main Configuration Structure

**Location**: `crates/core/src/config/mod.rs`

### Root Config Struct

```rust
pub struct Config {
  pub input: HashMap<String, String>,
  pub output: Box<OutputConfig>,
  pub root: String,
  pub mode: Mode,
  pub resolve: Box<ResolveConfig>,
  pub external: Vec<ConfigRegex>,
  pub define: HashMap<String, serde_json::Value>,
  pub runtime: Box<RuntimeConfig>,
  pub script: Box<ScriptConfig>,
  pub assets: Box<AssetsConfig>,
  pub css: Box<CssConfig>,
  pub html: Box<HtmlConfig>,
  pub sourcemap: Box<SourcemapConfig>,
  pub partial_bundling: Box<PartialBundlingConfig>,
  pub lazy_compilation: bool,
  pub core_lib_path: Option<String>,
  pub tree_shaking: Box<BoolOrObj<TreeShakingConfig>>,
  pub minify: Box<BoolOrObj<MinifyOptions>>,
  pub preset_env: Box<PresetEnvConfig>,
  pub record: bool,
  pub progress: bool,
  pub persistent_cache: Box<persistent_cache::PersistentCacheConfig>,
  pub concatenate_modules: bool,
  pub comments: Box<CommentsConfig>,
  pub custom: Box<HashMap<String, String>>,
}
```

## Configuration Sub-Modules

Located in `crates/core/src/config/`:

- `asset.rs` - AssetsConfig
- `css.rs` - CssConfig, CssModulesConfig, CssPrefixerConfig  
- `html.rs` - HtmlConfig
- `output.rs` - OutputConfig, ModuleFormatConfig
- `script.rs` - ScriptConfig
- `partial_bundling.rs` - PartialBundlingConfig
- `persistent_cache.rs` - PersistentCacheConfig
- `preset_env.rs` - PresetEnvConfig
- `tree_shaking.rs` - TreeShakingConfig
- `minify.rs` - MinifyOptions
- `external.rs` - ExternalConfig
- `comments.rs` - CommentsConfig

## Default Values

From `Config::default()`:
- `input`: `{"index": "./index.html"}`
- `root`: current directory
- `mode`: `Mode::Development`
- `lazy_compilation`: `true`
- `tree_shaking`: `true`
- `minify`: `true`
- `record`: `false`
- `progress`: `false`
- `concatenate_modules`: `false`

## Next Steps

- Review each sub-config module for detailed structure
- Analyze TypeScript equivalents in `packages/`
- Document breaking changes from v1 (if applicable)
