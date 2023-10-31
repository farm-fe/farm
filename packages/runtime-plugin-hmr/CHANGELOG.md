# @farmfe/runtime-plugin-hmr

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
