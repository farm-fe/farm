# @farmfe/runtime

## 0.12.0

### Minor Changes

- 966e2507: Optimize production size

## 0.11.2

### Patch Changes

- 58b256e2: runtime bundle

## 0.11.1

### Patch Changes

- 492353f8: fix: lazy compilation concurrency issue

## 0.11.0

### Minor Changes

- ef1b39bc: Top level await supported

## 0.10.0

### Minor Changes

- 8f8366de: Release Farm 1.0-beta

## 0.9.3

### Patch Changes

- 659244ed: Support create-farm-plugin and farm-plugin-tools

## 0.9.2

### Patch Changes

- 0ab4edf9: Fix failed to load external cjs require when output esm
- 0ab4edf9: throw error when dynamic load fail. close #836

## 0.9.1

### Patch Changes

- 736e6620: fix #878

## 0.9.0

### Minor Changes

- 24571102: Bump version

### Patch Changes

- 65c742c4: update template script && update runtime log

## 0.8.4

### Patch Changes

- 2bcf360e: fix #802

## 0.8.3

### Patch Changes

- dbecdf58: fix #769 and optimize cache

## 0.8.2

### Patch Changes

- c1a4fcc8: fix #747

## 0.8.1

### Patch Changes

- 6e88a1e3: update import.meta

## 0.8.0

### Minor Changes

- 0a20271a: Refactor render pot renders and optimize sourcemap generation

### Patch Changes

- c12156ff: Fix import.meta.url runtime issue

## 0.7.4

### Patch Changes

- 19447d7: set globalThis.require when initial module for UMD modules

## 0.7.3

### Patch Changes

- 62e6630: Fix error of runtime dynamic resource loading

## 0.7.2

### Patch Changes

- 509bac0: Fix that vite plugin is not compatible with Farm's lazy compilation

## 0.7.1

### Patch Changes

- Fix bugs that dev server should only try read local file system resources for images and fonts

## 0.7.0

### Minor Changes

- d604b5e: Support React SSR

## 0.6.2

### Patch Changes

- 3073e19: Isolate runtime from globalThis for script entries

## 0.6.1

### Patch Changes

- e0ff7f1: Using node cjs require to resolve external package

## 0.6.0

### Minor Changes

- a04008f: Support script, css and html minification

## 0.5.0

### Minor Changes

- Treat swc helpers as builtin module within runtime

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
