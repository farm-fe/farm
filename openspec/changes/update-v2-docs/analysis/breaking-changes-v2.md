# Farm v2 Breaking Changes Analysis

## Identified Breaking Changes

### From CHANGELOG.md Analysis

#### 1. HMR Server Origin Validation (v1.7.6)
**Breaking Change**: The HMR server now rejects all connections with unrecognized `Origin` headers.
- **Impact**: Clients need to update their configured ports and hosts if they want external apps to connect to the HMR server
- **Location**: Core HMR server implementation
- **Migration**: Update HMR client configuration with proper origin headers

#### 2. Config Structure Changes (v2.0.0-beta.5)
**Breaking Change**: All custom config moved from `config.custom` to corresponding config fields
- **Impact**: Config.custom.external → config.external (and similar for other custom configs)
- **Example**:
  ```typescript
  // v1
  config.custom.external = ['...']
  
  // v2
  config.external = ['...']
  ```
- **Migration**: Move all config.custom.* settings to their corresponding top-level config fields

#### 3. Major v2.0.0 Release
**Breaking Change**: Release of Farm v2.0.0  
- **Impact**: Multiple API and behavior changes
- **Location**: Across core, runtime, and plugin systems
- **Note**: Specific breaking changes need to be identified through detailed code review

## Configuration Breaking Changes

### Configuration Type Structure
Analysis from `crates/core/src/config/mod.rs`:

**Current v2 Config Structure**:
```rust
pub struct Config {
  pub input: HashMap<String, String>,              // Entry points
  pub output: Box<OutputConfig>,                   // Output configuration
  pub root: String,                                // Project root
  pub mode: Mode,                                  // development | production
  pub resolve: Box<ResolveConfig>,                 // Module resolution
  pub external: Vec<ConfigRegex>,                  // External dependencies
  pub define: HashMap<String, serde_json::Value>,  // Define constants
  pub runtime: Box<RuntimeConfig>,                 // Runtime config
  pub script: Box<ScriptConfig>,                   // Script processing
  pub assets: Box<AssetsConfig>,                   // Asset handling
  pub css: Box<CssConfig>,                         // CSS processing
  pub html: Box<HtmlConfig>,                       // HTML processing
  pub sourcemap: Box<SourcemapConfig>,             // Sourcemap config
  pub partial_bundling: Box<PartialBundlingConfig>, // Bundling strategy
  pub lazy_compilation: bool,                      // Lazy compilation
  pub core_lib_path: Option<String>,               // Core library path
  pub tree_shaking: Box<BoolOrObj<TreeShakingConfig>>, // Tree shaking
  pub minify: Box<BoolOrObj<MinifyOptions>>,       // Minification
  pub preset_env: Box<PresetEnvConfig>,            // Preset env
  pub record: bool,                                // Record stats
  pub progress: bool,                              // Show progress
  pub persistent_cache: Box<PersistentCacheConfig>, // Persistent cache
  pub concatenate_modules: bool,                   // Module concatenation
  pub comments: Box<CommentsConfig>,               // Comments config
  pub custom: Box<HashMap<String, String>>,        // Custom config (deprecated)
}
```

**Potential v1 → v2 Changes** (requires git history analysis):
- `custom.*` fields moved to top-level config
- Config field types may have changed
- Default values may have changed
- New required fields may have been added

## Plugin API Breaking Changes

### Rust Plugin API
From `crates/core/src/plugin/mod.rs` analysis:

**Current v2 Plugin Trait**: 35+ hooks including:
- Module processing hooks: `resolve`, `load`, `transform`, `parse`
- Module graph hooks: `module_graph_build_end`, `optimize_module_graph`
- Resource hooks: `render_resource_pot`, `generate_resources`
- HMR hooks: `update_modules`, `module_graph_updated`, `update_finished`

**Potential Changes** (requires comparison with v1):
- New hooks added
- Hook signatures may have changed
- Hook parameters may have changed
- Hook return types may have changed
- Hook execution order may have changed

### JavaScript Plugin API
From `packages/core/src/plugin/type.ts` analysis:

**Current v2 JsPlugin Interface**: Key hooks include:
- Configuration: `config`, `configResolved`, `configureServer`, `configureCompiler`
- Module processing: `resolve`, `load`, `transform`, `processModule`, `freezeModule`
- Resource processing: `renderStart`, `processRenderedResourcePot`, `augmentResourcePotHash`
- HTML: `transformHtml` (with order: pre/normal/post)
- HMR: `updateModules`, `updateFinished`

**Potential Changes** (requires comparison):
- Hook filter system may have changed
- CompilationContext API may have changed
- Hook callback signatures may have changed

## New Features in v2

From changelog analysis:

1. **UMD Format Support** (v2.0.0-beta.7)
   - `output.format=umd` now supported

2. **CSS Transform to Script** (v2.0.0-beta.7)
   - `css.transformToScript` option added

3. **Improved Vite Compatibility** (v2.0.0-beta.0)
   - Support for Vite plugin v6
   - Support for Vite Tailwind plugin v4

4. **Cache Improvements** (v2.0.0-beta.5)
   - JS cache API added
   - Module metadata cache added

5. **ASCII Only Output** (v1.7.9)
   - `output.asciiOnly` option added

## Required Analysis Tasks

To complete breaking changes documentation:

1. **Git History Review**: Compare v1.x tag with v2.0.0 tag for exact changes
2. **Config Comparison**: Compare v1 and v2 config type definitions
3. **Plugin API Comparison**: Compare v1 and v2 plugin interfaces
4. **Runtime Changes**: Identify runtime API changes
5. **CLI Changes**: Identify CLI flag or command changes
6. **Build Output Changes**: Document any build output format changes
7. **Dependency Updates**: Document major dependency updates that affect users

## Migration Priorities

### High Priority
1. Config structure changes (custom.* → top-level)
2. HMR origin validation
3. Plugin API changes (if any)

### Medium Priority
4. New config options with different defaults
5. Runtime API changes
6. CLI changes

### Low Priority  
7. Internal API changes
8. Performance optimizations (non-breaking)

## Next Steps

1. Review git commit history between v1.x and v2.0.0
2. Create detailed migration guide for each breaking change
3. Provide code examples showing v1 vs v2 patterns
4. Document workarounds or compatibility layers  
5. Add troubleshooting section for common migration issues
