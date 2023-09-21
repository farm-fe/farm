# @farmfe/core

## 0.12.7

### Patch Changes

- baec8bf: feat: add restartserver fn with update create-farm

## 0.12.6

### Patch Changes

- 407e3e2: Support more js plugin hooks

## 0.12.5

### Patch Changes

- 2d8635b: fix static assets loading issue when public path is empty

## 0.12.4

### Patch Changes

- b5f6e88: Fix that publicPath does not work for css and static assets

## 0.12.3

### Patch Changes

- be60085: update template version add host config ,Http server and websocket services share a set of ports

## 0.12.2

### Patch Changes

- 9381aa4: fix: host different cause websocket port error

## 0.12.1

### Patch Changes

- 5b1993f: Fix publicPath dev and support writeToDisk For dev server

## 0.12.0

### Minor Changes

- ad00276: Support sourcemap chain

## 0.11.1

### Patch Changes

- c8ef101: enable port auto-increment to prevent port conflict

## 0.11.0

### Minor Changes

- 56f235c: Upgrade swc crates and support emotion

## 0.10.7

### Patch Changes

- 75f58c1: Fix that extra watch file panic

## 0.10.6

### Patch Changes

- d6c3230: support add extra watch file

## 0.10.5

### Patch Changes

- b70ce32: Enlarge default watch debounce on windows
- Updated dependencies [b70ce32]
  - @farmfe/runtime-plugin-hmr@3.2.2

## 0.10.4

### Patch Changes

- 6aa7563: Optimize File Watcher - remove chokidar and introduce rust notify

## 0.10.3

### Patch Changes

- 8162eab: Remove hmr http request and use websocket/eval instead
- Updated dependencies [8162eab]
  - @farmfe/runtime-plugin-hmr@3.2.1

## 0.10.2

### Patch Changes

- 4fc704d: Fix that HMR middleware slow

## 0.10.1

### Patch Changes

- eb9f382: Fix script entry source map inject

## 0.10.0

### Minor Changes

- d604b5e: Support React SSR

### Patch Changes

- Updated dependencies [d604b5e]
  - @farmfe/runtime@0.7.0
  - @farmfe/runtime-plugin-hmr@3.2.0

## 0.9.10

### Patch Changes

- a40b07d: make resources order injected to html execution order

## 0.9.9

### Patch Changes

- 3073e19: Isolate runtime from globalThis for script entries
- Updated dependencies [3073e19]
  - @farmfe/runtime@0.6.2

## 0.9.8

### Patch Changes

- 596fc2a: Fix HMR patch_module_group_graph panic
- Updated dependencies [596fc2a]
  - @farmfe/runtime-plugin-hmr@3.2.0

## 0.9.7

### Patch Changes

- de18942: Fix that build env set to development

## 0.9.6

### Patch Changes

- c36c767: Do not resolve external dependencies when build farm.config.ts

## 0.9.5

### Patch Changes

- b92441b: Fix css hmr panic

## 0.9.4

### Patch Changes

- eb11635: Fix that css HMR will always reload the whole page

## 0.9.3

### Patch Changes

- 0e93bf0: Wait file write to finish by default

## 0.9.2

### Patch Changes

- 4656135: Fix hmr remove issue

## 0.9.1

### Patch Changes

- 7f0c8d7: Support `server.spa` option

## 0.9.0

### Minor Changes

- 55c0d0e: - Support configuring `html.base` to share html for multi-page application.
  - Support configuring `presetEnv.include`, `presetEnv.exclude`, `presetEnv.options` and `presetEnv.
  - Fix bug when editing html file, reload the page when html file change

### Patch Changes

- Updated dependencies [55c0d0e]
  - @farmfe/runtime-plugin-hmr@3.1.4

## 0.8.10

### Patch Changes

- ad90ff5: Support output.entryFilename and fix sass bugs

## 0.8.9

### Patch Changes

- f16ff29: Fix publicDir error when build

## 0.8.8

### Patch Changes

- 8f02078: Pretty syntax error for html, css and js/ts/jsx/tsx

## 0.8.7

### Patch Changes

- d8eeda9: Fix lazy compilation client and swc plugin deadlock

## 0.8.6

### Patch Changes

- 8a32a53: Support resolve @import and url() dependencies for css

## 0.8.5

### Patch Changes

- 4d719e4: Bugfix

## 0.8.4

### Patch Changes

- 3bb5808: Fix bugs:
  1. recognize immutable modules from config
  2. change all property on context to Box
  3. external, resolve, lazy compile and deadlock bugfix

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
