# @farmfe/core

## 0.15.3

### Patch Changes

- 18616c7d: Fix dev server random 503

## 0.15.2

### Patch Changes

- 09992927: Fix dev server random 503

## 0.15.1

### Patch Changes

- 295ec500: make vite plugins execute later than farm plugins

## 0.15.0

### Minor Changes

- 24571102: Bump version

### Patch Changes

- e4c9f81e: Break change: reset config and configResolved hooks hook functionality and structure
- e91a088a: resolve condition orderly
- 78c19574: support renderResourcePot/finalizeResources/augmentResourceHash/renderStart js hook
- 8846d063: Normalize js plugin hooks name
- Updated dependencies [65c742c4]
- Updated dependencies [24571102]
  - @farmfe/runtime@0.9.0
  - @farmfe/runtime-plugin-hmr@3.4.0
  - @farmfe/runtime-plugin-import-meta@0.1.0

## 0.14.18

### Patch Changes

- b181bb1c: support linux-x64-musl

## 0.14.17

### Patch Changes

- 329d37ef: fix enforceTargetMinSize panic

## 0.14.16

### Patch Changes

- 88a93d0a: Fix duplicate `/` in css url()

## 0.14.15

### Patch Changes

- 19c600d2: Fix win ia32 artifact not found
- 478c685f: update brand color update logger logic

## 0.14.14

### Patch Changes

- 0b022e7a: bug: PublicPath parsing error while setting server.open
- 418247f3: Optimize cache

## 0.14.13

### Patch Changes

- cfc5cfa2: Fix #814

## 0.14.12

### Patch Changes

- d533fa88: server: public path in open server error

## 0.14.11

### Patch Changes

- 0a83fff6: bump version Config hook undo
- 2bcf360e: fix #770
- Updated dependencies [2bcf360e]
  - @farmfe/runtime@0.8.4

## 0.14.10

### Patch Changes

- 9c6bb8bb: fix #787 #794 #795
- a6f7b165: bump version for publishing addtional cpu arch package

## 0.14.9

### Patch Changes

- 22752363: add clean command

## 0.14.8

### Patch Changes

- c6243c91: Fix immutable modules cache not found

## 0.14.7

### Patch Changes

- c231d824: Optimize persistent cache

## 0.14.6

### Patch Changes

- e38729de: fix import config cache
- 1c862451: fix #774 again

## 0.14.5

### Patch Changes

- 50db539e: support create https server & support restart
- 75c7018a: Fix #774

## 0.14.4

### Patch Changes

- dbecdf58: fix #769 and optimize cache
- b3a60e93: fix #768
- Updated dependencies [dbecdf58]
  - @farmfe/runtime-plugin-import-meta@0.0.4
  - @farmfe/runtime-plugin-hmr@3.3.1
  - @farmfe/runtime@0.8.3

## 0.14.3

### Patch Changes

- 18563f43: fix #761 static assets for internally supported html/js/ts/css modules
- 18563f43: Fix #760

## 0.14.2

### Patch Changes

- cf0dc914: Fix cjs export not found
- c1a4fcc8: fix #747
- Updated dependencies [c1a4fcc8]
  - @farmfe/runtime@0.8.2

## 0.14.1

### Patch Changes

- 6e88a1e3: bump version
- fc91c7df: Fix runtime resource panic
- Updated dependencies [6e88a1e3]
  - @farmfe/runtime-plugin-import-meta@0.0.3
  - @farmfe/runtime@0.8.1

## 0.14.0

### Minor Changes

- 72bfe2af: Support persistent cache and incremental building
- 0a20271a: Refactor render pot renders and optimize sourcemap generation

### Patch Changes

- c12156ff: fix #741
- Updated dependencies [72bfe2af]
- Updated dependencies [c12156ff]
- Updated dependencies [0a20271a]
  - @farmfe/runtime-plugin-hmr@3.3.0
  - @farmfe/runtime-plugin-import-meta@0.0.2
  - @farmfe/runtime@0.8.0

## 0.13.22

### Patch Changes

- 45761df: fix css resource pot load sourcemap

## 0.13.21

### Patch Changes

- 19447d7: support `export * from` for script entries and fix package.json browser resolve priority
- Updated dependencies [19447d7]
  - @farmfe/runtime@0.7.4

## 0.13.20

### Patch Changes

- bump version for #704

## 0.13.19

### Patch Changes

- a569977: Fix #693
- a569977: Optimize js plugin filters

## 0.13.18

### Patch Changes

- 219d91f: fix #691 #689 server error

## 0.13.17

### Patch Changes

- 35d294e: server host options default true
- 822c281: Fix #685
- Updated dependencies [35d294e]
  - @farmfe/runtime-plugin-hmr@3.2.5

## 0.13.16

### Patch Changes

- ddc3b40: fix enforceResources panic when lazy compile

## 0.13.15

### Patch Changes

- cc124a0: remove unless package rewrite logger and server banner

## 0.13.14

### Patch Changes

- 7e17e0b: Fix multiple exports issue when targetEnv is node

## 0.13.13

### Patch Changes

- 032bd4a: Fix bugs:
  1. `server.proxy` does not work as expected
  2. `plugin-css` should treat `xxx.png` as relative path
  3. `assets` like `/logo.png` under publicDir should be resolved to `publicDir/logo.png`

## 0.13.12

### Patch Changes

- 228ca7e: Support Vite-style `import.meta.glob`

## 0.13.11

### Patch Changes

- ac56943: Fix issues #652

## 0.13.10

### Patch Changes

- 0f93f94: Fix panic when scoped changed using vite plugin. #646

## 0.13.9

### Patch Changes

- 62e6630: Fix lazy compilation error when working with virtual module
- Updated dependencies [62e6630]
  - @farmfe/runtime@0.7.3

## 0.13.8

### Patch Changes

- 6a73829: fix: resolve exports filed resolve error

## 0.13.7

### Patch Changes

- preserve import.meta when targetEnv is Node

## 0.13.6

### Patch Changes

- 7daeb2a: Fix configure validation error and sass import resolve error

## 0.13.5

### Patch Changes

- db461dc: Fix tree-shake self-executed module issue && vite plugin adapter issue

## 0.13.4

### Patch Changes

- 0ee1751: Fix css modules sourcemap gen fail

## 0.13.3

### Patch Changes

- 6dd919e: Fix HMR Update fail when there are deep dependencies changed

## 0.13.2

### Patch Changes

- 509bac0: Fix that vite plugin is not compatible with Farm's lazy compilation
- Updated dependencies [509bac0]
- Updated dependencies [509bac0]
  - @farmfe/runtime@0.7.2
  - @farmfe/runtime-plugin-hmr@3.2.4

## 0.13.1

### Patch Changes

- Fix bugs that dev server should only try read local file system resources for images and fonts
- Updated dependencies
  - @farmfe/runtime@0.7.1

## 0.13.0

### Minor Changes

- f7b1b9d: Support vite plugins out of box
- 5be3aab: Implement RFC-003 New Partial Bundling Algorithm

## 0.12.11

### Patch Changes

- bcff2e8: format the normalizePublicPath function

## 0.12.10

### Patch Changes

- b44fde7: Support js plugin hook context methods for unplugin

## 0.12.9

### Patch Changes

- c12536a: fix: strictPort error

## 0.12.8

### Patch Changes

- 750ed61: preview command publicPath error

## 0.12.7

### Patch Changes

- baec8bf: feat: add restart-server fn with update create-farm

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
