# @farmfe/plugin-replace-dirname

## 2.0.0

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
