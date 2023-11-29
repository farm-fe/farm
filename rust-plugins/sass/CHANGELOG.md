# @farmfe/plugin-sass

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
