# @farmfe/runtime-plugin-hmr

## 3.5.3

### Patch Changes

- 89c40302: Support disable overlay
- de2c4821: Auto add `/` in socket URL if missing

## 3.5.2

### Patch Changes

- 492353f8: fix: lazy compilation concurrency issue

## 3.5.1

### Patch Changes

- Bump version

## 3.5.0

### Minor Changes

- 8f8366de: Release Farm 1.0-beta

## 3.4.2

### Patch Changes

- 659244ed: Support create-farm-plugin and farm-plugin-tools

## 3.4.1

### Patch Changes

- ea128f69: use log level with debug

## 3.4.0

### Minor Changes

- 24571102: Bump version

## 3.3.1

### Patch Changes

- dbecdf58: fix #769 and optimize cache

## 3.3.0

### Minor Changes

- 72bfe2af: Support persistent cache and incremental building
- 0a20271a: Refactor render pot renders and optimize sourcemap generation

### Patch Changes

- c12156ff: Fix import.meta.url runtime issue

## 3.2.5

### Patch Changes

- 35d294e: use windows in instead of import.meta

## 3.2.4

### Patch Changes

- 509bac0: Fix HMR runtime does not has 'data' property

## 3.2.3

### Patch Changes

- 54b93a9: Bump version

## 3.2.2

### Patch Changes

- b70ce32: Fix that vue SFC does not remove previous css styles when HMR

## 3.2.1

### Patch Changes

- 8162eab: Remove hmr http request and use websocket/eval instead

## 3.2.0

### Minor Changes

- 596fc2a: Support inject HMR port and host by config

## 3.1.4

### Patch Changes

- 55c0d0e: Reload the page when HMR fails

## 3.1.3

### Patch Changes

- 1148f68: Bugfix https://github.com/farm-fe/farm/issues/336

## 3.1.2

### Patch Changes

- 3dfc64f: 1. Fix hmr does not update lazy compiled module 2. Support sourcemap for Vue SFC

## 3.1.1

### Patch Changes

- 2ed0047: Fix that HMR does not remove css style

## 3.1.0

### Minor Changes

- a5364b5: Extract plugin react into a single plugin

## 3.0.5

### Patch Changes

- limit the watched files to optimize cold start speed and fix lazy compilation issue"

## 3.0.4

### Patch Changes

- Fix swc helper inject issue and optimize CLI

## 3.0.3

### Patch Changes

- 98d662f: feat: reconnect hmr server

## 3.0.2

### Patch Changes

- write resources to disk to optimize resources loading time

## 3.0.1

### Patch Changes

- Fix lazy compilation and partial bundling bugs
- Updated dependencies
  - @farmfe/runtime@0.3.1

## 3.0.0

### Patch Changes

- Updated dependencies [f915a35]
  - @farmfe/runtime@0.3.0

## 2.0.0

### Patch Changes

- Updated dependencies [e826221]
  - @farmfe/runtime@0.2.0

## 1.0.0

### Minor Changes

- 036aab6: Support react HMR

### Patch Changes

- Updated dependencies [036aab6]
  - @farmfe/runtime@0.1.0
