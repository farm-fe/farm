## ADDED Requirements

### Requirement: Quick start uses v2 syntax from actual implementation

The quick start guide SHALL use v2 syntax, APIs, and configuration based on actual v2 implementation code, not assumptions or v1 patterns.

#### Scenario: Installation instructions current
- **WHEN** user follows installation instructions
- **THEN** system provides v2 installation commands that actually work

#### Scenario: Initial project setup uses v2 config
- **WHEN** user creates first project following quick start
- **THEN** system shows configuration matching actual v2 config schema from code

#### Scenario: Code examples use v2 APIs
- **WHEN** user follows code examples in quick start
- **THEN** system provides examples using actual v2 API signatures from codebase

### Requirement: Quick start demonstrates working v2 example

The quick start guide SHALL provide a complete working example verified against v2 codebase.

#### Scenario: Example runs successfully
- **WHEN** user follows complete quick start example
- **THEN** system provides code that actually builds and runs with v2

#### Scenario: Example demonstrates core features
- **WHEN** user completes quick start
- **THEN** system has demonstrated core v2 features with working code

### Requirement: Quick start highlights v2 changes

The quick start guide SHALL include callouts highlighting what's different in v2 for users familiar with v1.

#### Scenario: V2 changes clearly marked
- **WHEN** v1 user reads quick start
- **THEN** system highlights where v2 differs from v1 with clear indicators

#### Scenario: Migration path referenced
- **WHEN** v1 user follows quick start
- **THEN** system references migration guide for complete v1→v2 details
