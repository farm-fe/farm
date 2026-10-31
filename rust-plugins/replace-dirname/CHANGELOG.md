# @farmfe/plugin-replace-dirname

## 2.0.0

### Patch Changes

- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

- ec5e740: Add reusable Farm Docs packages and Rust plugins for documentation planning, Markdown/MDX, static rendering, snippets, SEO and native styling. Migrate the official website to its independent Farm configuration with multilingual and versioned content.

  Add the Core output ownership, cache completion and runtime/HMR support needed by documentation builds and development updates. Preserve readable compiler errors in the CLI and expose the corresponding public types.

  Preserve license comments and their original source positions when combining ordinary or already hoisted module source maps.

### Earlier 2.0.0 preparation

### Patch Changes

- Pin the Rust SDK dependency family for rebuilding against core 3.0.0 and ABI 0.5.2 with nightly-2026-04-10. Native platform artifacts must be rebuilt and validated together with the matching core before publication.
- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

## 2.0.0-beta.1

### Patch Changes

- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

## 1.0.0-beta.0

### Major Changes

- f5ce9ea: Release Farm v2.0.0

## 1.0.0-nightly-20250827162746

### Patch Changes

- 9a227ad: Chore: release v2 nightly

## 1.0.0-nightly-20250411100807

### Patch Changes

- 60f40f0: bump nightly version

## 1.0.0-nightly-20241024075304

### Major Changes

- 9a1b2b9: bump replace-dirname plugin
