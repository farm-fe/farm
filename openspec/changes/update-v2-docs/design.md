## Context

Farm's documentation site is built with Docusaurus v3 located in `docs/docs/` with markdown files organized by topic. The current documentation reflects v1 APIs but needs comprehensive updates for v2 based on actual source code from:
- `crates/` - Rust implementation (compiler, plugins, core)
- `packages/` - JavaScript/TypeScript packages
- `js-plugins/` - Official JavaScript plugins
- `rust-plugins/` - Official Rust plugins

Current state:
- Documentation reflects v1 APIs and assumed behavior
- No code-based documentation approach
- Configuration docs don't match actual config types in code
- Plugin API docs are incomplete or outdated
- Feature docs (CSS, etc.) don't reflect actual plugin implementations
- No v2 migration guide exists

## Goals / Non-Goals

**Goals:**
- Migrate all current docs to v1 archive and rebuild for v2
- Base ALL documentation on actual source code analysis (code-first approach)
- Update configuration docs to match exact types from Rust/TypeScript implementations
- Document Rust and JavaScript plugin APIs from actual plugin interface code
- Update feature docs (CSS, preprocessors, etc.) based on actual plugin implementations in `js-plugins/`
- Create comprehensive v1→v2 migration guide with breaking changes
- Ensure all code examples are verified against actual v2 APIs

**Non-Goals:**
- Creating new documentation sections beyond what exists
- Translating documentation to other languages in this phase
- Updating example applications in `examples/` directory (reference only)
- Auto-generating docs from code comments (manual code review approach)
- Performance optimization of the docs site itself

## Decisions

**1. Code-First Documentation Approach**
- **Decision**: Base all documentation on actual source code analysis rather than assumptions or existing docs
- **Rationale**: Ensures accuracy; docs match implementation exactly; reduces bugs from doc-code mismatch
- **Alternative Considered**: Update existing docs incrementally without code review - rejected as it perpetuates inaccuracies

**2. V1 Documentation Archival Strategy**
- **Decision**: Archive current docs to v1 branch/version before creating v2 docs
- **Rationale**: Preserves existing docs for v1 users; clean slate for v2; clear version separation
- **Alternative Considered**: In-place updates with version markers - rejected as too complex to maintain

**3. Configuration Documentation from Types**
- **Decision**: Extract config documentation directly from TypeScript/Rust type definitions in codebase
- **Rationale**: Config types are source of truth; avoids drift between docs and actual config schema
- **Alternative Considered**: Manual config documentation - rejected due to maintenance burden and drift risk

**4. Plugin API Documentation Sources**
- **Decision**: Document plugin APIs by analyzing actual plugin interface code:
  - Rust: `crates/toolkit_plugin_types/`, `crates/macro_plugin/`
  - JavaScript: Plugin types in `packages/core/` and related packages
- **Rationale**: Plugin interfaces define the actual contract; documenting from code ensures accuracy
- **Alternative Considered**: High-level conceptual plugin docs - rejected as insufficient for plugin developers

**5. Feature Documentation from Plugin Implementations**
- **Decision**: Update feature docs (CSS, Tailwind, preprocessors) by reviewing actual plugin code in `js-plugins/`
- **Rationale**: Plugin implementations show exact capabilities and usage; prevents documenting unsupported features
- **Alternative Considered**: Generic feature descriptions - rejected as unhelpful for actual usage

**6. Migration Guide Placement**
- **Decision**: Create comprehensive `docs/migration/v1-to-v2.md` as primary migration resource
- **Rationale**: Centralizes breaking changes; easy to discover; can reference from other pages
- **Alternative Considered**: Scattered migration notes in each doc - rejected as hard to discover all changes

## Risks / Trade-offs

**[Risk]** Code analysis is time-consuming across large codebase
→ **Mitigation**: Prioritize high-impact areas (config, plugin APIs); use examples from `examples/` as validation

**[Risk]** Documentation may become too technical from code-first approach
→ **Mitigation**: Balance technical accuracy with user-friendly explanations; add conceptual overviews

**[Risk]** V1 users may be confused when docs suddenly change to v2
→ **Mitigation**: Prominent v1 archive link; clear version indicators; comprehensive migration guide

**[Risk]** Breaking changes may not be completely identified from code review alone
→ **Mitigation**: Cross-reference with git history; check for deprecation markers; review major PRs

**[Risk]** Plugin implementations may expose internal details not meant to be documented
→ **Mitigation**: Focus on public APIs and exported interfaces; verify what's intended for external use

**[Risk]** Documentation updates may lag actual v2 development
→ **Mitigation**: Coordinate with development team; prioritize stable APIs; mark experimental features

## Migration Plan

1. **Code Analysis Phase**: 
   - Audit `crates/` for config types, plugin interfaces, internal plugins and core APIs
   - Audit `packages/` for JavaScript APIs and type definitions
   - Audit `js-plugins/` and `rust-plugins` for plugin implementations and capabilities
   - Identify v1→v2 breaking changes from code and git history

2. **V1 Archive Setup**:
   - Create v1 documentation branch/version
   - Configure Docusaurus version switcher
   - Ensure v1 docs remain accessible

3. **V2 Documentation Creation**:
   - Create v1→v2 migration guide first
   - Update config documentation based on actual types
   - Document Rust plugin APIs from interface code
   - Document JavaScript plugin APIs from type definitions
   - Update feature docs (CSS, etc.) from plugin implementations
   - Update quick-start and API references

4. **Validation Phase**:
   - Verify all code examples against v2 codebase
   - Test that examples actually work
   - Review for consistency and completeness
   - Get technical review from core team

5. **Deployment**:
   - Deploy to staging for final review
   - Deploy to production
   - Announce v2 documentation to community

## Open Questions

- Which specific configuration types should be the primary source for config docs?
- Are there unstable/experimental APIs in v2 that should be marked differently?
- Should v1 docs be frozen completely or receive critical fixes?
- What's the source of truth for breaking changes (changelog, git history, code analysis)?
- How should internal vs public plugin APIs be distinguished?
- Are there example applications in `examples/` that demonstrate all v2 features?
