## 1. Codebase Analysis and Breaking Changes Identification

- [x] 1.1 Analyze `crates/` Rust code to identify configuration types and structures
- [x] 1.2 Analyze `packages/` TypeScript code to identify JavaScript API surface
- [x] 1.3 Review `crates/core/src/plugin` for Rust plugin interface definitions
- [x] 1.4 Review plugin type definitions in `packages/core/` for JavaScript plugin APIs
- [x] 1.5 Review `js-plugins/` and `rust-plugins` directory to catalog all official plugin implementations
- [x] 1.6 Compare v1 and v2 configs to identify breaking changes in config
- [x] 1.7 Compare v1 and v2 plugin interfaces to identify API changes
- [x] 1.8 Document all identified breaking changes in analysis notes

## 2. V1 Documentation Archive Setup

- [x] 2.1 Create v1 documentation version/branch using Docusaurus versioning
- [x] 2.2 Configure Docusaurus version switcher in docusaurus.config.js
- [x] 2.3 Add version indicators to documentation UI
- [x] 2.4 Test that v1 documentation is accessible via version switcher
- [x] 2.5 Add archived notice banner to v1 docs
- [ ] 2.6 Verify v1 docs are frozen and v2 is active branch

## 3. V2 Migration Guide Creation

- [x] 3.1 Create `docs/docs/migration/v1-to-v2.md` file
- [x] 3.2 Document configuration breaking changes from code analysis
- [ ] 3.3 Document plugin API breaking changes (Rust) from `crates/core/src/plugin`
- [ ] 3.4 Document plugin API breaking changes (JavaScript) from `packages/` analysis
- [ ] 3.5 Create API mapping table (v1 API → v2 API equivalents) from code
- [ ] 3.6 Add step-by-step migration instructions for common scenarios
- [x] 3.7 Add troubleshooting section for common migration issues
- [ ] 3.8 Document deprecated features with removal reasons and replacements
- [ ] 3.9 Add code examples showing v1 vs v2 patterns

## 4. Configuration Documentation from Code Analysis

- [ ] 4.1 Extract all configuration options from Rust type definitions in `crates/`
- [ ] 4.2 Extract all configuration options from TypeScript types in `packages/`
- [ ] 4.3 Document default values by analyzing implementation code
- [ ] 4.4 Rewrite `docs/docs/config/compilation-options.md` based on actual CompilationConfig types
- [ ] 4.5 Rewrite `docs/docs/config/dev-server.md` based on actual DevServerConfig types
- [ ] 4.6 Rewrite `docs/docs/config/shared-options.md` based on actual shared config types
- [ ] 4.7 Add type information (TypeScript/Rust types) to each config option
- [ ] 4.8 Add validation constraints found in code to documentation
- [ ] 4.9 Create verified working examples for each config section
- [ ] 4.10 Mark v2-specific options and deprecated v1 options

## 5. Rust Plugin API Documentation from Code

- [ ] 5.1 Analyze `crates/core/src/plugin` to extract all plugin trait definitions
- [ ] 5.2 Document each Rust plugin hook with exact signature from code
- [ ] 5.3 Analyze `crates/macro_plugin/` for plugin macro documentation
- [ ] 5.4 Document plugin lifecycle based on trait definitions
- [ ] 5.5 Document plugin context methods from trait definitions
- [ ] 5.6 Create/update Rust plugin API reference page
- [ ] 5.7 Add working code examples for each major plugin hook
- [ ] 5.8 Document plugin macro usage with examples
- [ ] 5.9 Add v1 to v2 Rust plugin migration section with exact changes

## 6. JavaScript Plugin API Documentation from Code

- [ ] 6.1 Analyze plugin type definitions in `packages/core/` for all interfaces
- [ ] 6.2 Document each JavaScript plugin hook with TypeScript signatures from code
- [ ] 6.3 Document plugin lifecycle methods from type definitions
- [ ] 6.4 Document plugin context API and all available methods
- [ ] 6.5 Create/update JavaScript plugin API reference page
- [ ] 6.6 Add working code examples for each major plugin hook
- [ ] 6.7 Document plugin configuration interface from types
- [ ] 6.8 Add v1 to v2 JavaScript plugin migration section with exact changes

## 7. CSS Documentation from Plugin Implementations

- [ ] 7.1 Analyze `js-plugins/@farmfe/plugin-tailwindcss` implementation
- [ ] 7.2 Update Tailwind CSS section in `docs/docs/features/css.md` based on plugin code
- [ ] 7.3 Document Tailwind plugin configuration options from plugin types
- [ ] 7.4 Analyze `js-plugins/@farmfe/plugin-sass` implementation
- [ ] 7.5 Update Sass section with configuration based on plugin code
- [ ] 7.6 Analyze `js-plugins/@farmfe/plugin-less` implementation
- [ ] 7.7 Update Less section with configuration based on plugin code
- [ ] 7.8 Analyze `js-plugins/@farmfe/plugin-postcss` implementation
- [ ] 7.9 Update PostCSS section with configuration based on plugin code
- [ ] 7.10 Update CSS modules documentation based on internal plugin implementation
- [ ] 7.11 Add verified working examples for each CSS preprocessor
- [ ] 7.12 Update installation commands with correct package names

## 8. Feature Documentation from Internal Plugin Implementations

- [ ] 8.1 Analyze `crates/plugin_html/` implementation
- [ ] 8.2 Update HTML feature docs based on actual plugin capabilities
- [ ] 8.3 Analyze `crates/plugin_static_assets/` implementation
- [ ] 8.4 Update static assets documentation based on plugin code
- [ ] 8.5 Analyze `crates/plugin_json/` implementation
- [ ] 8.6 Update JSON handling documentation based on actual implementation
- [ ] 8.7 Review `crates/plugin_script/` for JavaScript/TypeScript handling
- [ ] 8.8 Update script handling documentation
- [ ] 8.9 Review other internal plugins in `crates/plugin_*`
- [ ] 8.10 Update documentation for additional features found in plugins

## 9. Quick Start Guide Update

- [ ] 9.1 Verify installation commands work with v2
- [ ] 9.2 Update `docs/docs/quick-start.mdx` with v2 installation
- [ ] 9.3 Update project initialization example with v2 config syntax from code
- [ ] 9.4 Update basic configuration example using actual v2 config types
- [ ] 9.5 Update first build example with v2 API calls
- [ ] 9.6 Add working complete example verified against v2 codebase
- [ ] 9.7 Add "What's new in v2" callout with highlights
- [ ] 9.8 Add reference to migration guide for v1 users
- [ ] 9.9 Test all quick start examples actually work with v2

## 10. V2 Feature Highlights Documentation

- [ ] 10.1 Create or update v2 feature highlights section
- [ ] 10.2 Document new v2 features identified from code analysis
- [ ] 10.3 Document improved v2 features with technical details
- [ ] 10.4 Add performance improvements with metrics if available
- [ ] 10.5 Document enhanced plugin system capabilities
- [ ] 10.6 Create v1 vs v2 comparison table
- [ ] 10.7 Mark features as "New in v2", "Improved in v2", "Removed in v2"
- [ ] 10.8 Add migration recommendations for each major change

## 11. JavaScript API Reference Update

- [ ] 11.1 Analyze JavaScript API exports from `packages/core/src/index.ts`
- [ ] 11.2 Update core API documentation with exact signatures from code
- [ ] 11.3 Add TypeScript type definitions to all API documentation
- [ ] 11.4 Review and update HMR API documentation from actual implementation
- [ ] 11.5 Review and update Runtime API documentation from code
- [ ] 11.6 Update build API documentation with v2 changes
- [ ] 11.7 Add working code examples for each major API section
- [ ] 11.8 Mark deprecated APIs and show v2 replacements
- [ ] 11.9 Document new v2-only APIs discovered in code

## 12. Code Examples Verification and Update

- [ ] 12.1 Create list of all code examples across documentation
- [ ] 12.2 Update code examples to use v2 syntax from actual APIs
- [ ] 12.3 Add explanatory comments highlighting v2 changes
- [ ] 12.4 Verify syntax correctness of all examples
- [ ] 12.5 Test critical examples actually compile/run with v2
- [ ] 12.6 Update framework examples (React, Vue, etc.) with v2 patterns
- [ ] 12.7 Reference working examples from `examples/` directory where applicable
- [ ] 12.8 Ensure all imports and API calls match actual v2 exports

## 13. Documentation Consistency and Quality

- [ ] 13.1 Review all updated documentation for technical accuracy
- [ ] 13.2 Ensure consistent terminology across all documentation
- [ ] 13.3 Run link checker and fix broken internal links
- [ ] 13.4 Add v2 version indicators throughout documentation
- [ ] 13.5 Ensure code blocks have proper language syntax highlighting
- [ ] 13.6 Update frontmatter/metadata for all changed pages
- [ ] 13.7 Review for consistent code style in examples
- [ ] 13.8 Ensure all type references are accurate to code

## 14. Testing and Validation

- [ ] 14.1 Build documentation site locally and check for errors
- [ ] 14.2 Test version switcher functionality between v1 and v2
- [ ] 14.3 Validate all internal links work correctly
- [ ] 14.4 Test external links are still valid
- [ ] 14.5 Verify critical code examples in actual v2 environment
- [ ] 14.6 Check that all config examples are valid per v2 schema
- [ ] 14.7 Review navigation and ensure logical flow
- [ ] 14.8 Get technical review from core team on accuracy

## 15. Deployment and Release

- [ ] 15.1 Update Docusaurus config to set v2 as default version
