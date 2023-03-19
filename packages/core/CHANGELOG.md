# @farmfe/core

## 0.5.0

### Minor Changes

- 9987627: Queue async update and wait for compiling to finish when refresh
- bd8c762: Change query from HashMap to Vec<(String, String)> to make it's order stable
- a5364b5: Extract plugin react into a single plugin

### Patch Changes

- Updated dependencies [a5364b5]
  - @farmfe/runtime-plugin-hmr@3.1.0
  - @farmfe/runtime@0.4.0

## 0.4.7

### Patch Changes

- f137492: Make query part of ModuleId

## 0.4.6

### Patch Changes

- Fix update log

## 0.4.5

### Patch Changes

- Auto external node native module when reading farm.config.ts
- Updated dependencies
  - @farmfe/runtime@0.3.4

## 0.4.4

### Patch Changes

- Fix that file watcher does not work properly when add or remove dependencies

## 0.4.3

### Patch Changes

- 61f5fbe: Support sourcemap for source files

## 0.4.2

### Patch Changes

- limit the watched files to optimize cold start speed and fix lazy compilation issue"
- Updated dependencies
  - @farmfe/runtime-plugin-hmr@3.0.5
  - @farmfe/runtime@0.3.3

## 0.4.1

### Patch Changes

- Fix swc helper inject issue and optimize CLI
- Updated dependencies
  - @farmfe/runtime-plugin-hmr@3.0.4

## 0.4.0

### Minor Changes

- 4ee1260: Support resolve `browser` field in package.json
- 835e06b: Support resolve, load and transform hook for js plugins
- 9d07b4d: Support static assets and define

### Patch Changes

- Updated dependencies [98d662f]
  - @farmfe/runtime-plugin-hmr@3.0.3

## 0.3.3

### Patch Changes

- Optimize disk usage

## 0.3.2

### Patch Changes

- write resources to disk to optimize resources loading time
- Updated dependencies
  - @farmfe/runtime-plugin-hmr@3.0.2
  - @farmfe/runtime@0.3.2

## 0.3.1

### Patch Changes

- Fix lazy compilation and partial bundling bugs
- Updated dependencies
  - @farmfe/runtime-plugin-hmr@3.0.1
  - @farmfe/runtime@0.3.1

## 0.3.0

### Minor Changes

- f915a35: Support lazy compilation and partial bundling

  - remove resource pot graph to optimize the compilation speed
  - implement partial bundling algorithm
  - optimize @farmfe/cli, remove @farmfe/core from its dependencies
  - optimize plugin react to skip duplicate module building based on process.env.NODE_ENV

### Patch Changes

- Updated dependencies [f915a35]
  - @farmfe/runtime@0.3.0
  - @farmfe/runtime-plugin-hmr@3.0.0

## 0.2.0

### Minor Changes

- e826221: Support css HMR and dynamic resource compiling and loading for dynamic import

### Patch Changes

- Updated dependencies [e826221]
  - @farmfe/runtime@0.2.0
  - @farmfe/runtime-plugin-hmr@2.0.0

## 0.1.5

### Patch Changes

- Fix GLIBC_2.32 not found on linux

## 0.1.4

### Patch Changes

- Fix windows config resolve error

## 0.1.3

### Patch Changes

- Fix that binary dependencies do not exist

## 0.1.2

### Patch Changes

- Bump version

## 0.1.1

### Patch Changes

- Publish native optional dependencies

## 0.1.0

### Minor Changes

- 036aab6: Support react HMR

### Patch Changes

- Updated dependencies [036aab6]
  - @farmfe/runtime-plugin-hmr@1.0.0
  - @farmfe/runtime@0.1.0
