# @farmfe/core

## 0.8.3

### Patch Changes

- e0ff7f1: Fix module system detect and sort alias by length
- Updated dependencies [e0ff7f1]
  - @farmfe/runtime@0.6.1

## 0.8.2

### Patch Changes

- 1148f68: - Support lazy compilation for vue and support moduleTypes filter for js plugin transform hook
  - Bundle @farmfe/js-plugin-vue with Farm
- Updated dependencies [1148f68]
  - @farmfe/runtime-plugin-hmr@3.1.3

## 0.8.1

### Patch Changes

- 6f97c87: - Add missing dependencies execa
  - Add ./ to config.input when the values of config.input is not absolute path and do not start with ./
  - Alias resolve take precedent over all other resolve strategies
  - Do not resolve html dependencies starts with `http` and `/`

## 0.8.0

### Minor Changes

- e780747: Support swc plugin

## 0.7.6

### Patch Changes

- a281ce6: optimize core script code

## 0.7.5

### Patch Changes

- 086d1a3: Fix bugs when transforming css

## 0.7.4

### Patch Changes

- 1c42307: Fix that css module should be execution order

## 0.7.3

### Patch Changes

- d9fe509: add polyfill config

## 0.7.2

### Patch Changes

- 659bc72: css modules support path hash & css sourcemap
- e0521e5: support css prefixer
- 926c9cb: Fix css modules HMR & update ci yaml

## 0.7.1

### Patch Changes

- 079bb21: Fix that zod parse js plugin executor lead to napi error

## 0.7.0

### Minor Changes

- a04008f: Support script, css and html minification

### Patch Changes

- 7ff4d97: support css modules
- Updated dependencies [a04008f]
  - @farmfe/runtime@0.6.0
  - @farmfe/runtime-plugin-hmr@3.1.2

## 0.6.4

### Patch Changes

- 6fa3454: Do not resolve browser when targetEnv is node

## 0.6.3

### Patch Changes

- Support resolve .. and absolute dir

## 0.6.2

### Patch Changes

- Updated dependencies
  - @farmfe/runtime@0.5.0
  - @farmfe/runtime-plugin-hmr@3.1.2

## 0.6.1

### Patch Changes

- c45470e: Fix that tree shake does not ignore non-script modules when in production mode

## 0.6.0

### Minor Changes

- 9838407: Support tree shake

### Patch Changes

- cc94e33: support entry key as resource name

## 0.5.4

### Patch Changes

- 6371e96: fix ModuleType serialization
- c4d9c95: support .json file compile

## 0.5.3

### Patch Changes

- 3dfc64f: 1. Fix hmr does not update lazy compiled module 2. Support sourcemap for Vue SFC
- Updated dependencies [3dfc64f]
  - @farmfe/runtime-plugin-hmr@3.1.2
  - @farmfe/runtime@0.4.2

## 0.5.2

### Patch Changes

- 2ed0047: Fix that HMR does not remove css style
- Updated dependencies [2ed0047]
  - @farmfe/runtime-plugin-hmr@3.1.1
  - @farmfe/runtime@0.4.1

## 0.5.1

### Patch Changes

- Fix rustPluginResolver error on windows

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
