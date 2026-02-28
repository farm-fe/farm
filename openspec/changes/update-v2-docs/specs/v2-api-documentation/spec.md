## ADDED Requirements

### Requirement: Complete v2 API documentation from source code

The documentation SHALL provide comprehensive API reference for all Farm v2 JavaScript and Rust APIs based on actual code in `crates/` and `packages/`, including type definitions, function signatures, and usage examples.

#### Scenario: User searches for specific API function
- **WHEN** user searches the API reference
- **THEN** system displays complete documentation including exact signature from code, parameters, return type, and examples

#### Scenario: User views type definitions
- **WHEN** user navigates to API documentation
- **THEN** system displays TypeScript interface definitions extracted from `packages/` source with all properties and their types

#### Scenario: User finds code examples for API usage
- **WHEN** user reads API documentation
- **THEN** system displays practical code examples verified against v2 codebase

### Requirement: Rust plugin API documentation from crate interfaces

The documentation SHALL document the v2 Rust plugin API based on actual plugin interfaces from `crates/toolkit_plugin_types/` and `crates/macro_plugin/`, including all hooks, lifecycle methods, and plugin context methods.

#### Scenario: User develops Rust plugin
- **WHEN** user reads the Rust plugin API documentation
- **THEN** system displays all available hooks from `crates/toolkit_plugin_types/` with exact signatures and examples

#### Scenario: User learns plugin lifecycle in Rust
- **WHEN** user follows the Rust plugin documentation
- **THEN** system shows lifecycle methods from actual plugin trait definitions

#### Scenario: User uses plugin macros
- **WHEN** user reads macro documentation
- **THEN** system shows macro usage from `crates/macro_plugin/` with examples

### Requirement: JavaScript plugin API documentation from package types

The documentation SHALL clearly document the v2 JavaScript plugin system API based on plugin type definitions in `packages/core/` and related packages, including hooks, lifecycle methods, and plugin context methods.

#### Scenario: User develops JavaScript plugin
- **WHEN** user reads the JS plugin API documentation
- **THEN** system displays all available hooks with exact TypeScript signatures from source

#### Scenario: User migrates v1 plugin to v2
- **WHEN** user follows the plugin API documentation
- **THEN** system shows which v1 hooks were renamed or replaced in v2 based on code analysis

#### Scenario: User accesses plugin context
- **WHEN** user reads plugin context documentation
- **THEN** system shows all context methods from actual type definitions in packages

### Requirement: Configuration schema reference from code types

The documentation SHALL provide complete reference for all v2 configuration options with types and default values extracted from actual configuration type definitions in `crates/` (Rust) and `packages/` (TypeScript).

#### Scenario: User configures Farm
- **WHEN** user reads the config reference
- **THEN** system displays all available options from code types organized by section with explanations and defaults

#### Scenario: User finds config option type information
- **WHEN** user searches for a specific config option
- **THEN** system shows exact type from code, usage examples, and default value from implementation
