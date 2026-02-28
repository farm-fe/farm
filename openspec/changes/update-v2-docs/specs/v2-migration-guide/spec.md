## ADDED Requirements

### Requirement: Comprehensive v1 to v2 migration guide

The documentation SHALL provide a comprehensive guide at `docs/migration/v1-to-v2.md` to migrate from Farm v1 to v2, covering all breaking changes identified from code analysis, deprecations, and new configuration options.

#### Scenario: User identifies breaking changes
- **WHEN** user reads the migration guide
- **THEN** system displays a complete list of breaking changes organized by component (config, plugins, APIs) extracted from actual code differences

#### Scenario: User learns migration steps
- **WHEN** user follows the migration guide
- **THEN** system provides step-by-step instructions for updating v1 code to v2 based on actual API changes

#### Scenario: User finds API mapping reference
- **WHEN** user searches for a v1 API change
- **THEN** system shows the v2 equivalent with exact function signatures from code and any required modifications

### Requirement: Breaking changes identified from codebase

The migration guide SHALL document breaking changes identified through analysis of `crates/` and `packages/` code, comparing v1 and v2 implementations.

#### Scenario: Config breaking changes documented
- **WHEN** user reads config migration section
- **THEN** system shows exact config type changes from Rust/TypeScript definitions

#### Scenario: Plugin API breaking changes documented
- **WHEN** user reads plugin migration section
- **THEN** system shows exact plugin interface changes from `crates/toolkit_plugin_types/`

### Requirement: Troubleshooting section for v2 migration issues

The documentation SHALL include a troubleshooting guide addressing common migration problems identified during code analysis and their solutions.

#### Scenario: User encounters migration error
- **WHEN** user searches for a specific error message
- **THEN** system displays the troubleshooting section with the solution based on actual error types

#### Scenario: User needs plugin migration help
- **WHEN** user reads the migration guide
- **THEN** system provides specific guidance for migrating custom plugins to v2 with code examples

### Requirement: Deprecation notices in documentation

The documentation SHALL clearly mark deprecated v1 features found in code and link users to their v2 replacements.

#### Scenario: User views documentation for deprecated feature
- **WHEN** user reads a page about a feature deprecated in v2
- **THEN** system displays a notice indicating the feature is deprecated with exact replacement from code
