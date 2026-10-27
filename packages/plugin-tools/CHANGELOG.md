# @farmfe/plugin-tools

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

- 6e4352c: Publish native plugin packages with public access so first-time scoped package releases succeed.
- 46bf9fd: Support passing a custom Rust build profile through `farm-plugin-tools build` without also forwarding the default `--release` flag.
- Updated dependencies [ec5e740]
- Updated dependencies [47d569d]
  - @farmfe/utils@2.0.0

### Earlier 2.0.0 preparation

### Minor Changes

- Add a prepare command that previews platform versions and optional dependencies without publishing. Use prepare --write to apply the reviewed manifest changes after validating all platform package names. Binary content, packing and installation remain separate acceptance steps.

### Patch Changes

- Correct Node engines to match the installed dependency ranges: core and CLI require Node 18 or Node 20+, while Rust plugin build tools require Node 22.20 or Node 24.12+. Runtime and native build-tool compatibility are separate. Node 16 and 19 are excluded from core/CLI support; Node 18/20 and early Node 22 are excluded from the build tools.
- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

- 6e4352c: Publish native plugin packages with public access so first-time scoped package releases succeed.
- 46bf9fd: Support passing a custom Rust build profile through `farm-plugin-tools build` without also forwarding the default `--release` flag.
- Updated dependencies [47d569d]
  - @farmfe/utils@2.0.0

## 2.0.0-beta.3

### Patch Changes

- 6e4352c: Publish native plugin packages with public access so first-time scoped package releases succeed.

## 2.0.0-beta.2

### Patch Changes

- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

- 46bf9fd: Support passing a custom Rust build profile through `farm-plugin-tools build` without also forwarding the default `--release` flag.

## 2.0.0-beta.1

### Patch Changes

- Updated dependencies [47d569d]
  - @farmfe/utils@2.0.0-beta.1

## 2.0.0-beta.0

### Major Changes

- f5ce9ea: Release Farm v2.0.0

### Patch Changes

- Updated dependencies [f5ce9ea]
  - @farmfe/utils@2.0.0-beta.0

## 0.1.2-nightly-20250827162746

### Patch Changes

- 9a227ad: Chore: release v2 nightly
- Updated dependencies [9a227ad]
  - @farmfe/utils@1.0.0-nightly-20250827162746

## 0.1.2-nightly-20241022124925

### Patch Changes

- Updated dependencies [6a1038c]
  - @farmfe/utils@1.0.0-nightly-20241022124925

## 0.1.1

### Patch Changes

- ecc176af: fix: dependencies

## 0.1.0

### Minor Changes

- d330af58: Bump version

## 0.0.5

### Patch Changes

- Fix rust plugin prepublish

## 0.0.4

### Patch Changes

- Fix plugin tools log

## 0.0.3

### Patch Changes

- fix copy artifacts

## 0.0.2

### Patch Changes

- 90f0bd5f: fix plugin-tools execution error

## 0.0.1

### Patch Changes

- 659244ed: Support create-farm-plugin and farm-plugin-tools
