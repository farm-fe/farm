# Farm v2 Codebase Analysis - Rust Plugin Interface

## Main Plugin Trait

**Location**: `crates/core/src/plugin/mod.rs`

### Plugin Trait Definition

```rust
pub trait Plugin: Any + Send + Sync {
  fn name(&self) -> &str;
  fn priority(&self) -> i32 { DEFAULT_PRIORITY }
  
  // Configuration hooks
  fn config(&self, _config: &mut Config) -> Result<Option<()>>;
  fn plugin_cache_loaded(&self, _cache: &Vec<u8>, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  
  // Build lifecycle hooks
  fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn build_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn finish(&self, _stat: &Stats, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  
  // Module resolution and loading hooks
  fn resolve(&self, _param: &PluginResolveHookParam, _context: &Arc<CompilationContext>, _hook_context: &PluginHookContext) -> Result<Option<PluginResolveHookResult>>;
  fn load(&self, _param: &PluginLoadHookParam, _context: &Arc<CompilationContext>, _hook_context: &PluginHookContext) -> Result<Option<PluginLoadHookResult>>;
  fn transform(&self, _param: &PluginTransformHookParam, _context: &Arc<CompilationContext>) -> Result<Option<PluginTransformHookResult>>;
  
  // Module processing hooks
  fn parse(&self, _param: &PluginParseHookParam, _context: &Arc<CompilationContext>, _hook_context: &PluginHookContext) -> Result<Option<ModuleMetaData>>;
  fn process_module(&self, _param: &mut PluginProcessModuleHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn analyze_deps(&self, _param: &mut PluginAnalyzeDepsHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn finalize_module(&self, _param: &mut PluginFinalizeModuleHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn freeze_module(&self, _param: &mut PluginFreezeModuleHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  
  // Module graph hooks  
  fn module_graph_build_end(&self, _module_graph: &mut ModuleGraph, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn optimize_module_graph(&self, _module_graph: &mut ModuleGraph, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn freeze_module_graph_meta(&self, _module_graph: &mut ModuleGraph, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn analyze_module_graph(&self, _module_graph: &mut ModuleGraph, _context: &Arc<CompilationContext>, _hook_context: &PluginHookContext) -> Result<Option<ModuleGroupGraph>>;
  
  // Resource generation hooks
  fn generate_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn partial_bundling(&self, _modules: &Vec<ModuleId>, _context: &Arc<CompilationContext>, _hook_context: &PluginHookContext) -> Result<Option<Vec<ResourcePot>>>;
  fn process_resource_pots(&self, _resource_pots: &mut Vec<&mut ResourcePot>, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  
  // Resource rendering hooks
  fn render_start(&self, _config: &Config, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn render_resource_pot(&self, _resource_pot: &ResourcePot, _context: &Arc<CompilationContext>, _hook_context: &PluginHookContext) -> Result<Option<ResourcePotMetaData>>;
  fn process_rendered_resource_pot(&self, _resource_pot: &mut ResourcePot, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn augment_resource_pot_hash(&self, _render_pot: &ResourcePot, _context: &Arc<CompilationContext>) -> Result<Option<String>>;
  fn optimize_resource_pot(&self, _resource: &mut ResourcePot, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn generate_resources(&self, _resource_pot: &mut ResourcePot, _context: &Arc<CompilationContext>, _hook_context: &PluginHookContext) -> Result<Option<PluginGenerateResourcesHookResult>>;
  fn process_generated_resources(&self, _resources: &mut PluginGenerateResourcesHookResult, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn handle_entry_resource(&self, _resource: &mut PluginHandleEntryResourceHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn finalize_resources(&self, _param: &mut PluginFinalizeResourcesHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn generate_end(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  
  // HMR/Update hooks
  fn update_modules(&self, _params: &mut PluginUpdateModulesHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn module_graph_updated(&self, _param: &PluginModuleGraphUpdatedHookParam, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  fn update_finished(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>>;
  
  // Cache hooks
  fn handle_persistent_cached_module(&self, _module: &Module, _context: &Arc<CompilationContext>) -> Result<Option<bool>>;
  fn write_plugin_cache(&self, _context: &Arc<CompilationContext>) -> Result<Option<Vec<u8>>>;
}
```

## Plugin Hook Context

```rust
pub struct PluginHookContext {
  pub caller: Option<String>,
  pub meta: HashMap<String, String>
}
```

## Key Hook Parameter Types

Located in `crates/core/src/plugin/hooks/`:

- `PluginResolveHookParam` / `PluginResolveHookResult` - Module resolution
- `PluginLoadHookParam` / `PluginLoadHookResult` - Module loading
- `PluginTransformHookParam` / `PluginTransformHookResult` - Module transformation
- `PluginAnalyzeDepsHookParam` - Dependency analysis
- `PluginProcessModuleHookParam` - Module processing
- `PluginFinalizeModuleHookParam` - Module finalization
- `PluginFreezeModuleHookParam` - Module freezing
- `PluginHandleEntryResourceHookParam` - Entry resource handling
- `PluginGenerateResourcesHookResult` - Resource generation
- `PluginFinalizeResourcesHookParam` - Resource finalization
- `PluginUpdateModulesHookParam` / `UpdateResult` / `UpdateType` - HMR updates
- `PluginModuleGraphUpdatedHookParam` - Module graph updates

## Plugin Lifecycle Order

1. **Config Phase**: `config`, `plugin_cache_loaded`
2. **Build Start**: `build_start`
3. **Module Resolution**: `resolve`, `load`, `transform`
4. **Module Processing**: `parse`, `process_module`, `analyze_deps`, `finalize_module`, `freeze_module`
5. **Module Graph**: `module_graph_build_end`, `optimize_module_graph`, `freeze_module_graph_meta`, `analyze_module_graph`
6. **Build End**: `build_end`
7. **Resource Generation**: `generate_start`, `partial_bundling`, `process_resource_pots`
8. **Resource Rendering**: `render_start`, `render_resource_pot`, `process_rendered_resource_pot`, `augment_resource_pot_hash`, `optimize_resource_pot`
9. **Resource Finalization**: `generate_resources`, `process_generated_resources`, `handle_entry_resource`, `finalize_resources`
10. **Generation End**: `generate_end`
11. **Completion**: `finish`

## HMR Lifecycle

1. `update_modules` - Called when modules are updated
2. `module_graph_updated` - Called after module graph is updated
3. `update_finished` - Called after update is complete

## Default Priority

`DEFAULT_PRIORITY = 100`

## Next Steps

- Document macro_plugin features
- Compare with v1 plugin interface (if applicable)
- Document hook execution order and dependencies
