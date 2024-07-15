# Changelog

## 1.1.0

### Minor Changes

- 966e2507: Bump version cause core changed

### Patch Changes

- 385e5b25: feat: normalize usage of rust plugins and js plugins

## 1.0.5

### Patch Changes

- 6a9b13c2: chore(rust-plugins): fix ts warning

## 1.0.4

### Patch Changes

- 61f702e5: chore: remove unless code

## 1.0.3

### Patch Changes

- 1ec7dd74: fix resolve path

## 1.0.2

### Patch Changes

- cb7df71f: Support alias resolve and url rebase for import.meta.glob, sass and less plugins

## 1.0.1

### Patch Changes

- a749b5af: Fix Vite project migrations issues

## 1.0.0

### Major Changes

- 8f8366de: Release 1.0.0-beta

### Patch Changes

- 56dbe2c0: Fix prepublishOnly script

## 1.0.0-beta.1

### Patch Changes

- 56dbe2c0: Fix prepublishOnly script
  All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_plugin_sass-v0.0.1) - 2024-03-13

### Added

- support linux-x64-musl ([#838](https://github.com/farm-fe/farm/pull/838))
- support add extra watch file ([#470](https://github.com/farm-fe/farm/pull/470))
- support embed sass ([#161](https://github.com/farm-fe/farm/pull/161))

### Fixed

- vite migrate bugs ([#912](https://github.com/farm-fe/farm/pull/912))
- [#909](https://github.com/farm-fe/farm/pull/909) ([#914](https://github.com/farm-fe/farm/pull/914))
- win32-ia32 ci build ([#826](https://github.com/farm-fe/farm/pull/826))
- fix [#760](https://github.com/farm-fe/farm/pull/760) [#761](https://github.com/farm-fe/farm/pull/761) ([#765](https://github.com/farm-fe/farm/pull/765))
- linux arm64 native plugin resolving ([#750](https://github.com/farm-fe/farm/pull/750))
- [#693](https://github.com/farm-fe/farm/pull/693) ([#695](https://github.com/farm-fe/farm/pull/695))
- bugs when migrate from vite to farm ([#665](https://github.com/farm-fe/farm/pull/665))
- [#660](https://github.com/farm-fe/farm/pull/660) incorrect default path resolve for default sass embedded binary ([#664](https://github.com/farm-fe/farm/pull/664))
- sass @import alias and panic when scoped changed using vite plug… ([#648](https://github.com/farm-fe/farm/pull/648))
- config validation error ([#628](https://github.com/farm-fe/farm/pull/628))
- css modules sourcemap gen fail ([#621](https://github.com/farm-fe/farm/pull/621))
- extra watch files panic and dev server hook execution order ([#486](https://github.com/farm-fe/farm/pull/486))
- the import file is not found in sass ([#290](https://github.com/farm-fe/farm/pull/290))
- pnpm-v8 depsversion error ([#223](https://github.com/farm-fe/farm/pull/223))

### Other

- Version Packages (beta) ([#1016](https://github.com/farm-fe/farm/pull/1016))
- release ([#1009](https://github.com/farm-fe/farm/pull/1009))
- update rust plugin scripts
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- Version Packages ([#906](https://github.com/farm-fe/farm/pull/906))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- Version Packages ([#843](https://github.com/farm-fe/farm/pull/843))
- Version Packages ([#840](https://github.com/farm-fe/farm/pull/840))
- Version Packages ([#817](https://github.com/farm-fe/farm/pull/817))
- Version Packages ([#797](https://github.com/farm-fe/farm/pull/797))
- Version Packages ([#793](https://github.com/farm-fe/farm/pull/793))
- downgrade glibc and support win32-ia32 and win32-arm64 ([#792](https://github.com/farm-fe/farm/pull/792))
- Version Packages ([#764](https://github.com/farm-fe/farm/pull/764))
- update rust plugins scripts ([#759](https://github.com/farm-fe/farm/pull/759))
- Version Packages ([#751](https://github.com/farm-fe/farm/pull/751))
- Version Packages ([#732](https://github.com/farm-fe/farm/pull/732))
- linux arm64 support ([#743](https://github.com/farm-fe/farm/pull/743))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Version Packages ([#697](https://github.com/farm-fe/farm/pull/697))
- bump version for react and sass ([#676](https://github.com/farm-fe/farm/pull/676))
- Version Packages ([#666](https://github.com/farm-fe/farm/pull/666))
- Version Packages ([#649](https://github.com/farm-fe/farm/pull/649))
- Version Packages ([#629](https://github.com/farm-fe/farm/pull/629))
- Version Packages ([#624](https://github.com/farm-fe/farm/pull/624))
- Version Packages ([#617](https://github.com/farm-fe/farm/pull/617))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Version Packages ([#612](https://github.com/farm-fe/farm/pull/612))
- Version Packages ([#501](https://github.com/farm-fe/farm/pull/501))
- Version Packages ([#487](https://github.com/farm-fe/farm/pull/487))
- Version Packages ([#479](https://github.com/farm-fe/farm/pull/479))
- Version Packages ([#452](https://github.com/farm-fe/farm/pull/452))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- Version Packages ([#414](https://github.com/farm-fe/farm/pull/414))
- Version Packages ([#393](https://github.com/farm-fe/farm/pull/393))
- Version Packages ([#384](https://github.com/farm-fe/farm/pull/384))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- Version Packages ([#371](https://github.com/farm-fe/farm/pull/371))
- Version Packages ([#358](https://github.com/farm-fe/farm/pull/358))
- Version Packages ([#330](https://github.com/farm-fe/farm/pull/330))
- Version Packages ([#284](https://github.com/farm-fe/farm/pull/284))
- upgrade zig and napi-cli
- Version Packages ([#243](https://github.com/farm-fe/farm/pull/243))
- Version Packages ([#188](https://github.com/farm-fe/farm/pull/188))
- solve issues when add dependencies in HMR ([#194](https://github.com/farm-fe/farm/pull/194))
- Feat/sass options ([#196](https://github.com/farm-fe/farm/pull/196))
- prepublishOnly ([#180](https://github.com/farm-fe/farm/pull/180))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_plugin_sass-v0.0.1) - 2024-03-12

### Added

- support linux-x64-musl ([#838](https://github.com/farm-fe/farm/pull/838))
- support add extra watch file ([#470](https://github.com/farm-fe/farm/pull/470))
- support embed sass ([#161](https://github.com/farm-fe/farm/pull/161))

### Fixed

- vite migrate bugs ([#912](https://github.com/farm-fe/farm/pull/912))
- [#909](https://github.com/farm-fe/farm/pull/909) ([#914](https://github.com/farm-fe/farm/pull/914))
- win32-ia32 ci build ([#826](https://github.com/farm-fe/farm/pull/826))
- fix [#760](https://github.com/farm-fe/farm/pull/760) [#761](https://github.com/farm-fe/farm/pull/761) ([#765](https://github.com/farm-fe/farm/pull/765))
- linux arm64 native plugin resolving ([#750](https://github.com/farm-fe/farm/pull/750))
- [#693](https://github.com/farm-fe/farm/pull/693) ([#695](https://github.com/farm-fe/farm/pull/695))
- bugs when migrate from vite to farm ([#665](https://github.com/farm-fe/farm/pull/665))
- [#660](https://github.com/farm-fe/farm/pull/660) incorrect default path resolve for default sass embedded binary ([#664](https://github.com/farm-fe/farm/pull/664))
- sass @import alias and panic when scoped changed using vite plug… ([#648](https://github.com/farm-fe/farm/pull/648))
- config validation error ([#628](https://github.com/farm-fe/farm/pull/628))
- css modules sourcemap gen fail ([#621](https://github.com/farm-fe/farm/pull/621))
- extra watch files panic and dev server hook execution order ([#486](https://github.com/farm-fe/farm/pull/486))
- the import file is not found in sass ([#290](https://github.com/farm-fe/farm/pull/290))
- pnpm-v8 depsversion error ([#223](https://github.com/farm-fe/farm/pull/223))

### Other

- update rust plugin scripts
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- Version Packages ([#906](https://github.com/farm-fe/farm/pull/906))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- Version Packages ([#843](https://github.com/farm-fe/farm/pull/843))
- Version Packages ([#840](https://github.com/farm-fe/farm/pull/840))
- Version Packages ([#817](https://github.com/farm-fe/farm/pull/817))
- Version Packages ([#797](https://github.com/farm-fe/farm/pull/797))
- Version Packages ([#793](https://github.com/farm-fe/farm/pull/793))
- downgrade glibc and support win32-ia32 and win32-arm64 ([#792](https://github.com/farm-fe/farm/pull/792))
- Version Packages ([#764](https://github.com/farm-fe/farm/pull/764))
- update rust plugins scripts ([#759](https://github.com/farm-fe/farm/pull/759))
- Version Packages ([#751](https://github.com/farm-fe/farm/pull/751))
- Version Packages ([#732](https://github.com/farm-fe/farm/pull/732))
- linux arm64 support ([#743](https://github.com/farm-fe/farm/pull/743))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Version Packages ([#697](https://github.com/farm-fe/farm/pull/697))
- bump version for react and sass ([#676](https://github.com/farm-fe/farm/pull/676))
- Version Packages ([#666](https://github.com/farm-fe/farm/pull/666))
- Version Packages ([#649](https://github.com/farm-fe/farm/pull/649))
- Version Packages ([#629](https://github.com/farm-fe/farm/pull/629))
- Version Packages ([#624](https://github.com/farm-fe/farm/pull/624))
- Version Packages ([#617](https://github.com/farm-fe/farm/pull/617))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Version Packages ([#612](https://github.com/farm-fe/farm/pull/612))
- Version Packages ([#501](https://github.com/farm-fe/farm/pull/501))
- Version Packages ([#487](https://github.com/farm-fe/farm/pull/487))
- Version Packages ([#479](https://github.com/farm-fe/farm/pull/479))
- Version Packages ([#452](https://github.com/farm-fe/farm/pull/452))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- Version Packages ([#414](https://github.com/farm-fe/farm/pull/414))
- Version Packages ([#393](https://github.com/farm-fe/farm/pull/393))
- Version Packages ([#384](https://github.com/farm-fe/farm/pull/384))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- Version Packages ([#371](https://github.com/farm-fe/farm/pull/371))
- Version Packages ([#358](https://github.com/farm-fe/farm/pull/358))
- Version Packages ([#330](https://github.com/farm-fe/farm/pull/330))
- Version Packages ([#284](https://github.com/farm-fe/farm/pull/284))
- upgrade zig and napi-cli
- Version Packages ([#243](https://github.com/farm-fe/farm/pull/243))
- Version Packages ([#188](https://github.com/farm-fe/farm/pull/188))
- solve issues when add dependencies in HMR ([#194](https://github.com/farm-fe/farm/pull/194))
- Feat/sass options ([#196](https://github.com/farm-fe/farm/pull/196))
- prepublishOnly ([#180](https://github.com/farm-fe/farm/pull/180))

# @farmfe/plugin-sass

## 1.0.0-beta.0

### Major Changes

- 8f8366de: Release 1.0.0-beta

## 0.5.0

### Minor Changes

- 7fc2a650: Support preserving comments for Js/Ts/Jsx/Tsx modules

### Patch Changes

- 069249a1: Fix #909

## 0.4.0

### Minor Changes

- 24571102: Bump version

## 0.3.5

### Patch Changes

- b181bb1c: support linux-x64-musl

## 0.3.4

### Patch Changes

- cfc5cfa2: Bump version as core changed

## 0.3.3

### Patch Changes

- a6f7b165: bump version for publishing addtional cpu arch package

## 0.3.2

### Patch Changes

- 18563f43: Fix #760

## 0.3.1

### Patch Changes

- 13c326ae: Fix arm64 arch resolving

## 0.3.0

### Minor Changes

- 72bfe2af: Support persistent cache and incremental building
- 0a20271a: Refactor render pot renders and optimize sourcemap generation

## 0.2.9

### Patch Changes

- a569977: Optimize js plugin filters

## 0.2.8

### Patch Changes

- Bump version as Farm core changed

## 0.2.7

### Patch Changes

- 032bd4a: Fix bugs:
  1. `server.proxy` does not work as expected
  2. `plugin-css` should treat `xxx.png` as relative path
  3. `assets` like `/logo.png` under publicDir should be resolved to `publicDir/logo.png`

## 0.2.6

### Patch Changes

- Fix incorrect sass binary resolve for default path

## 0.2.5

### Patch Changes

- 0f93f94: Fix alias @import fail

## 0.2.4

### Patch Changes

- 7daeb2a: Fix configure validation error and sass import resolve error

## 0.2.3

### Patch Changes

- 0ee1751: Fix css modules sourcemap gen fail

## 0.2.2

### Patch Changes

- 509bac0: Fix that vite plugin is not compatible with Farm's lazy compilation

## 0.2.1

### Patch Changes

- 5be3aab: Bump version due to update of @farmfe/core

## 0.2.0

### Minor Changes

- 56f235c: Upgrade swc crates and support emotion

## 0.1.2

### Patch Changes

- 75f58c1: Fix that extra watch file panic

## 0.1.1

### Patch Changes

- d6c3230: support add extra watch file

## 0.1.0

### Minor Changes

- d604b5e: Support React SSR

## 0.0.10

### Patch Changes

- eb11635: Fix that css HMR will always reload the whole page

## 0.0.9

### Patch Changes

- 55c0d0e: Bump version as core changed

## 0.0.8

### Patch Changes

- ad90ff5: Support output.entryFilename and fix sass bugs

## 0.0.7

### Patch Changes

- d8eeda9: bump version as core changed

## 0.0.6

### Patch Changes

- 3bb5808: Bump version as core changed

## 0.0.5

### Patch Changes

- e780747: Bump version cause the Rust core changed

## 0.0.4

### Patch Changes

- 926c9cb: Fix css modules HMR & update ci yaml

## 0.0.3

### Patch Changes

- 6fa3454: Do not resolve browser when targetEnv is node

## 0.0.2

### Patch Changes

- 611beb3: Support sass options
