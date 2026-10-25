# @farmfe/plugin-mdx

## 2.0.0

### Patch Changes

- Pin the Rust SDK dependency family for rebuilding against core 3.0.0 and ABI 0.5.2 with nightly-2026-04-10. Native platform artifacts must be rebuilt and validated together with the matching core before publication.
- 5cf6af9: Fix Rust plugin build scripts and tooling compatibility with napi-rs CLI option parsing.

  - Add backward compatibility in plugin-tools by stripping legacy `--cargo-name` before invoking `@napi-rs/cli`.
  - Update Rust plugin build scripts to use `-p <cargo-package>` without the unsupported `--cargo-name` flag.
  - Fix `@farmfe/plugin-react` `build:publish` script to target the correct Cargo package.
  - Update create-farm-plugin Rust template and docs examples to generate the corrected build command.

- Resolve plugin options declarations with explicit JavaScript extensions so strict NodeNext consumers can load their types. Include the virtual and YAML options declarations in published packages, and apply the same import convention to the Rust plugin scaffold.

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

## 0.0.7

### Patch Changes

- update ci config to release packages

## 0.0.6

### Patch Changes

- fix package import path

## 0.0.5

### Patch Changes

- rename package name

## 0.0.4

### Patch Changes

- chore: update farm version

## 0.0.3

### Patch Changes

- feat: update farm version

## 0.0.2

### Patch Changes

- feat(rust-plugin): optimization plugin

## 0.0.1

### Patch Changes

- add mdx plugins
