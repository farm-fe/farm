## Why

Farm's documentation is critical for user adoption and developer experience. As the project matures and v2 approaches, the existing documentation needs comprehensive updates to reflect new features, breaking changes, and API improvements. Without updated documentation, users will struggle to migrate to v2 and understand the new capabilities.

## What Changes

- **Migrate current docs to v1**: Archive current documentation as v1 and establish new v2 documentation structure
- **Update docs based on actual code**: Review and document functionality from `crates/` (Rust) and `packages/` (JavaScript) implementations
- **Configuration documentation overhaul**: Update all configuration documentation to match actual configuration types and implementation in the codebase
- **Feature documentation updates**: Update feature docs (CSS, HTML, static assets, etc.) based on actual plugin implementations in `js-plugins/` directory and all internal plugins
  - Update Tailwind CSS support documentation based on `@farmfe/plugin-tailwindcss`
  - Update CSS preprocessor docs based on actual plugin implementations
- **Plugin API documentation**: Fully document Rust and JavaScript plugin APIs based on actual plugin interfaces
  - Rust plugin API from `crates/plugin_*` and `crates/core`
  - JavaScript plugin API from plugin type definitions in `packages/`
- **Add migration guide for v2**: Comprehensive guide for migrating from v1 to v2 with breaking changes
- **Update code examples**: All code examples in docs must reflect v2 APIs and actual working syntax

## Capabilities

### New Capabilities

- `v1-docs-migration`: Migrate existing documentation to v1 branch/version and establish v2 documentation structure
- `v2-migration-guide`: Comprehensive guide helping users migrate from v1 to v2, documenting breaking changes and deprecations
- `v2-api-documentation`: Complete API documentation for v2 based on actual code in `crates/` and `packages/`
- `v2-feature-highlights`: New documentation highlighting v2-specific features and improvements
- `code-based-config-docs`: Update configuration documentation to match actual configuration types and implementation

### Modified Capabilities

- `quick-start`: Update quick start guide to use v2 syntax and configuration based on actual v2 implementation
- `config-reference`: Rewrite configuration documentation based on actual config types in the codebase
- `plugin-development`: Complete rewrite of plugin API documentation based on actual plugin interfaces from Rust (`crates/core/src/plugin`) and JS (`packages/`) code
- `css-features`: Update CSS/Tailwind/preprocessor documentation based on actual plugins in `js-plugins/` (@farmfe/plugin-tailwindcss, etc.)

## Impact

- **Documentation Site**: All pages under `docs/docs/` will be migrated to v1 and rewritten for v2
- **Configuration Reference**: Must align with actual types from `crates/` configuration implementation
- **Feature Documentation**: 
  - CSS/preprocessor docs based on `js-plugins/@farmfe/plugin-sass`, `@farmfe/plugin-less`, `@farmfe/plugin-postcss`, `@farmfe/plugin-tailwindcss`
  - HTML docs based on `crates/plugin_html/`
  - Static assets based on `crates/plugin_static_assets/`
  - Other features based on actual plugin implementations
- **API References**: 
  - Rust plugin API from `crates/core/src/plugin`, `crates/macro_plugin/`
  - JavaScript plugin API from plugin type definitions in `packages/core/` and related packages
- **Code Examples**: Examples in `examples/` directory may serve as reference for documentation examples
- **User Experience**: Accurate, code-based documentation will reduce confusion and support burden
