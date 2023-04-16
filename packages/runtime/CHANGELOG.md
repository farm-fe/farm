# @farmfe/runtime

## 0.5.0

### Minor Changes

- Treat swc helplers as builtin module within runtime

## 0.4.2

### Patch Changes

- 3dfc64f: 1. Fix hmr does not update lazy compiled module 2. Support sourcemap for Vue SFC

## 0.4.1

### Patch Changes

- 2ed0047: Fix that HMR does not remove css style

## 0.4.0

### Minor Changes

- a5364b5: Extract plugin react into a single plugin

## 0.3.4

### Patch Changes

- Auto external node native module when reading farm.config.ts

## 0.3.3

### Patch Changes

- limit the watched files to optimize cold start speed and fix lazy compilation issue"

## 0.3.2

### Patch Changes

- write resources to disk to optimize resources loading time

## 0.3.1

### Patch Changes

- Fix lazy compilation and partial bundling bugs

## 0.3.0

### Minor Changes

- f915a35: Support lazy compilation and partial bundling

  - remove resource pot graph to optimize the compilation speed
  - implement partial bundling algorithm
  - optimize @farmfe/cli, remove @farmfe/core from its dependencies
  - optimize plugin react to skip duplicate module building based on process.env.NODE_ENV

## 0.2.0

### Minor Changes

- e826221: Support css HMR and dynamic resource compiling and loading for dynamic import

## 0.1.0

### Minor Changes

- 036aab6: Support react HMR
