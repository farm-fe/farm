# @farmfe/plugin-auto-import

## 2.0.0-beta.1

### Patch Changes

- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

## 2.0.0-beta.0

### Major Changes

- release farm v2-beta

## 0.1.0-beta.0

### Minor Changes

- feat(rust-plugins): update farm v2-beta

## 0.0.9

### Patch Changes

- feat: auto import support inject at end

## 0.0.8

### Patch Changes

- fix: scan_dirs_exports exprot all

## 0.0.7

### Patch Changes

- rename package name

## 0.0.6

### Patch Changes

- chore: update farm version

## 0.0.5

### Patch Changes

- feat: update farm version

## 0.0.4

### Patch Changes

- update package.json

## 0.0.3

### Patch Changes

- fix(auto-import): alias import preset

## 0.0.2

### Patch Changes

- fix: options type

## 0.0.1

### Patch Changes

- feat: @farmfe/plugin-auto-import
