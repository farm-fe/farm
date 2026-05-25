# @farmfe/plugin-worker

## 2.0.0

### Patch Changes

- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

- 5cf6af9: Fix inline worker resource patching when generated JavaScript resources contain non-UTF8 bytes, preventing worker placeholders from being erased before replacement.

## 2.0.0-beta.1

### Patch Changes

- Fix invalid worker output extension

## 2.0.0-beta.0

### Major Changes

- release farm v2-beta

## 0.1.0-beta.0

### Minor Changes

- feat(rust-plugins): update farm v2-beta

## 0.0.10

### Patch Changes

- rename package name

## 0.0.8

### Patch Changes

- chore: update farm version

## 0.0.7

### Patch Changes

- feat: update farm version

## 0.0.6

### Patch Changes

- feat: worker\*path suport /src/\*\*/\_ path

## 0.0.5

### Patch Changes

- feat: worker suport constructor

## 0.0.4

### Patch Changes

- persistentCache build file is empty

## 0.0.3

### Patch Changes

- chore: priority & inline option

## 0.0.2

### Patch Changes

- feat(rust-plugin): optimization plugin

## 0.0.1

### Patch Changes

- feat(rust-plugins): plugin worker
