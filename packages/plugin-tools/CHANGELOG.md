# @farmfe/plugin-tools

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
