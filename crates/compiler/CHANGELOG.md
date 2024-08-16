# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.12](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.11...farmfe_compiler-v0.0.12) - 2024-08-16

### Fixed
- bundle import namespace name uniq ([#1696](https://github.com/farm-fe/farm/pull/1696))
- single bundle ([#1653](https://github.com/farm-fe/farm/pull/1653))

## [0.0.11](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.10...farmfe_compiler-v0.0.11) - 2024-07-25

### Added
- support compiler.traceModuleGraph ([#1654](https://github.com/farm-fe/farm/pull/1654))

## [0.0.10](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.9...farmfe_compiler-v0.0.10) - 2024-07-19

### Other
- release ([#1612](https://github.com/farm-fe/farm/pull/1612))

## [0.0.9](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.8...farmfe_compiler-v0.0.9) - 2024-05-28

### Added
- support obj external & dts support resolvedPaths ([#1282](https://github.com/farm-fe/farm/pull/1282))

### Fixed
- cache issues ([#1301](https://github.com/farm-fe/farm/pull/1301))

## [0.0.8](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.7...farmfe_compiler-v0.0.8) - 2024-05-09

### Added
- support lazy compilation when targeting node ([#1035](https://github.com/farm-fe/farm/pull/1035))
- support top level await ([#1202](https://github.com/farm-fe/farm/pull/1202))

### Fixed
- circle module require ([#1290](https://github.com/farm-fe/farm/pull/1290))

### Other
- Fix/lazy compilation ([#1253](https://github.com/farm-fe/farm/pull/1253))
- Version Packages v1.1.0 ([#1214](https://github.com/farm-fe/farm/pull/1214))
- update swc to v0.90 ([#1227](https://github.com/farm-fe/farm/pull/1227))

## [0.0.7](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.6...farmfe_compiler-v0.0.7) - 2024-04-13

### Other
- Fix/lazy compile mixed import ([#1175](https://github.com/farm-fe/farm/pull/1175))

## [0.0.6](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.5...farmfe_compiler-v0.0.6) - 2024-04-11

### Fixed
- sass import sourcemap ([#1154](https://github.com/farm-fe/farm/pull/1154))

## [0.0.5](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.4...farmfe_compiler-v0.0.5) - 2024-04-09

### Other
- updated the following local packages: farmfe_plugin_static_assets

## [0.0.4](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.3...farmfe_compiler-v0.0.4) - 2024-04-08

### Added
- improve tree shake traverse ([#1118](https://github.com/farm-fe/farm/pull/1118))

## [0.0.3](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.2...farmfe_compiler-v0.0.3) - 2024-04-01

### Fixed
- less/sass url rebase and url publicPath ([#1071](https://github.com/farm-fe/farm/pull/1071))

## [0.0.2](https://github.com/farm-fe/farm/compare/farmfe_compiler-v0.0.1...farmfe_compiler-v0.0.2) - 2024-03-24

### Added
- minify modules instead of resource pots ([#1025](https://github.com/farm-fe/farm/pull/1025))
- remove if (import.meta.hot) guard for production ([#1030](https://github.com/farm-fe/farm/pull/1030))

### Fixed
- add remove import meta hot condition ([#1056](https://github.com/farm-fe/farm/pull/1056))
- tree shake variable assign ([#1054](https://github.com/farm-fe/farm/pull/1054))
- treeshake class decl assign ([#1038](https://github.com/farm-fe/farm/pull/1038))

### Other
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_compiler-v0.0.1) - 2024-03-12

### Added
- eliminate more useless code ([#971](https://github.com/farm-fe/farm/pull/971))
- add progress plugin ([#948](https://github.com/farm-fe/farm/pull/948))
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- *(hmr)* refactor hmr ([#835](https://github.com/farm-fe/farm/pull/835))
- Supoort import.meta.url and import.meta.env ([#738](https://github.com/farm-fe/farm/pull/738))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- *(refactor)* RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- Support js plugin hook context methods for unplugin ([#589](https://github.com/farm-fe/farm/pull/589))
- feat support sourcemap chain based on swc sourcemap ([#528](https://github.com/farm-fe/farm/pull/528))
- support add extra watch file ([#470](https://github.com/farm-fe/farm/pull/470))
- support resolve @import and url() dependencies for css ([#367](https://github.com/farm-fe/farm/pull/367))
- support swc plugins ([#199](https://github.com/farm-fe/farm/pull/199))
- *(watch)* WIP add watch command ([#313](https://github.com/farm-fe/farm/pull/313))
- css module config schema & sourcemap ([#281](https://github.com/farm-fe/farm/pull/281))
- support polyfill ([#255](https://github.com/farm-fe/farm/pull/255))
- support css modules ([#230](https://github.com/farm-fe/farm/pull/230))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- feat entry key as resource name ([#205](https://github.com/farm-fe/farm/pull/205))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))
- support parse json file ([#162](https://github.com/farm-fe/farm/pull/162))
- queue async update and wait for compiling to finish when refresh ([#97](https://github.com/farm-fe/farm/pull/97))
- add string when generate css id and change query HashMap to Vec… ([#90](https://github.com/farm-fe/farm/pull/90))
- auto external node native module when using farm.config.ts ([#81](https://github.com/farm-fe/farm/pull/81))
- support sourcemap ([#77](https://github.com/farm-fe/farm/pull/77))
- add load and transform hook for js plugins ([#58](https://github.com/farm-fe/farm/pull/58))
- serve resources with dev server ([#21](https://github.com/farm-fe/farm/pull/21))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- implement the basic compilation flow ([#17](https://github.com/farm-fe/farm/pull/17))
- setup node binding and tests ([#8](https://github.com/farm-fe/farm/pull/8))
- init project with cargo and pnpm

### Fixed
- vue bugs ([#973](https://github.com/farm-fe/farm/pull/973))
- [#952](https://github.com/farm-fe/farm/pull/952) ([#959](https://github.com/farm-fe/farm/pull/959))
- [#941](https://github.com/farm-fe/farm/pull/941) ([#945](https://github.com/farm-fe/farm/pull/945))
- vite migrate bugs ([#912](https://github.com/farm-fe/farm/pull/912))
- failed to load external cjs require when output esm ([#892](https://github.com/farm-fe/farm/pull/892))
- [#878](https://github.com/farm-fe/farm/pull/878) ([#881](https://github.com/farm-fe/farm/pull/881))
- [#850](https://github.com/farm-fe/farm/pull/850) ([#870](https://github.com/farm-fe/farm/pull/870))
- error logging and remove process exit ([#852](https://github.com/farm-fe/farm/pull/852))
- enforceTargetMinSize panic ([#833](https://github.com/farm-fe/farm/pull/833))
- css url asset ([#827](https://github.com/farm-fe/farm/pull/827))
- win32-ia32 ci build ([#826](https://github.com/farm-fe/farm/pull/826))
- [#814](https://github.com/farm-fe/farm/pull/814) ([#816](https://github.com/farm-fe/farm/pull/816))
- [#787](https://github.com/farm-fe/farm/pull/787) [#794](https://github.com/farm-fe/farm/pull/794) [#785](https://github.com/farm-fe/farm/pull/785) ([#800](https://github.com/farm-fe/farm/pull/800))
- [#774](https://github.com/farm-fe/farm/pull/774) ([#777](https://github.com/farm-fe/farm/pull/777))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- [#768](https://github.com/farm-fe/farm/pull/768) ([#771](https://github.com/farm-fe/farm/pull/771))
- fix [#760](https://github.com/farm-fe/farm/pull/760) [#761](https://github.com/farm-fe/farm/pull/761) ([#765](https://github.com/farm-fe/farm/pull/765))
- issue 747 ([#758](https://github.com/farm-fe/farm/pull/758))
- css resource cannot find sourcemap ([#714](https://github.com/farm-fe/farm/pull/714))
- bugs ([#710](https://github.com/farm-fe/farm/pull/710))
- [#685](https://github.com/farm-fe/farm/pull/685) ([#687](https://github.com/farm-fe/farm/pull/687))
- bugs when migrate from vite to farm ([#665](https://github.com/farm-fe/farm/pull/665))
- issue [#652](https://github.com/farm-fe/farm/pull/652) ([#655](https://github.com/farm-fe/farm/pull/655))
- sass @import alias and panic when scoped changed using vite plug… ([#648](https://github.com/farm-fe/farm/pull/648))
- css modules sourcemap gen fail ([#621](https://github.com/farm-fe/farm/pull/621))
- HMR update fail when there are deep dependencies changed ([#619](https://github.com/farm-fe/farm/pull/619))
- lazy compialtion error and windows css error ([#454](https://github.com/farm-fe/farm/pull/454))
- Isolate runtime from globalThis for script entries ([#446](https://github.com/farm-fe/farm/pull/446))
- hmr patch ([#443](https://github.com/farm-fe/farm/pull/443))
- mode set to development when build ([#439](https://github.com/farm-fe/farm/pull/439))
- css hmr will always reload the whole page ([#413](https://github.com/farm-fe/farm/pull/413))
- hmr remove dependency ([#405](https://github.com/farm-fe/farm/pull/405))
- *(WIP)* resolve module_graph  error ([#355](https://github.com/farm-fe/farm/pull/355))
- vue migrate bugs ([#357](https://github.com/farm-fe/farm/pull/357))
- module system detection bug ([#345](https://github.com/farm-fe/farm/pull/345))
- tree shake should ignore shake non-script modules ([#210](https://github.com/farm-fe/farm/pull/210))
- coverage ci ([#176](https://github.com/farm-fe/farm/pull/176))
- lazy compilation and partial bundling bug ([#44](https://github.com/farm-fe/farm/pull/44))

### Other
- publish crates
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- support absolute path for script tag ([#927](https://github.com/farm-fe/farm/pull/927))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- switch to chokidar ([#886](https://github.com/farm-fe/farm/pull/886))
- Fix transformIndexHtml does not work as expected ([#884](https://github.com/farm-fe/farm/pull/884))
- Feat/rollup hook compatible ([#842](https://github.com/farm-fe/farm/pull/842))
- optimize cache 4 ([#820](https://github.com/farm-fe/farm/pull/820))
- optimize cache 3 ([#786](https://github.com/farm-fe/farm/pull/786))
- optimize cache 2 ([#785](https://github.com/farm-fe/farm/pull/785))
- *(persistent-cache)* optimize cache ([#782](https://github.com/farm-fe/farm/pull/782))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Fix/js plugins filters ([#678](https://github.com/farm-fe/farm/pull/678))
- normalzie js plugins options ([#668](https://github.com/farm-fe/farm/pull/668))
- Fix/bugs ([#640](https://github.com/farm-fe/farm/pull/640))
- Fix tree-shake self-executed module issue && vite plugin adapterissue ([#626](https://github.com/farm-fe/farm/pull/626))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Feat/js plugin adaptor ([#613](https://github.com/farm-fe/farm/pull/613))
- update swc and support emotion ([#500](https://github.com/farm-fe/farm/pull/500))
- *(*)* apply some lint suggestions ([#474](https://github.com/farm-fe/farm/pull/474))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- add x-data-spreadsheet example ([#422](https://github.com/farm-fe/farm/pull/422))
- Fix/css hmr ([#419](https://github.com/farm-fe/farm/pull/419))
- add more features ([#387](https://github.com/farm-fe/farm/pull/387))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- pretty syntax error ([#372](https://github.com/farm-fe/farm/pull/372))
- Optimize tree shake perf ([#369](https://github.com/farm-fe/farm/pull/369))
- solving bugs ([#338](https://github.com/farm-fe/farm/pull/338))
- support sync option when update ([#315](https://github.com/farm-fe/farm/pull/315))
- Feat/resource module order ([#312](https://github.com/farm-fe/farm/pull/312))
- make the modules order be execution order ([#311](https://github.com/farm-fe/farm/pull/311))
- update css modules hmr and ci yaml ([#299](https://github.com/farm-fe/farm/pull/299))
- Feat/css prefixer ([#294](https://github.com/farm-fe/farm/pull/294))
- format with prettier ([#266](https://github.com/farm-fe/farm/pull/266))
- do not resolve browser when target env is node ([#238](https://github.com/farm-fe/farm/pull/238))
- bugfix/source-module-graph-error ([#192](https://github.com/farm-fe/farm/pull/192))
- add profiler and optimize resolve speed ([#217](https://github.com/farm-fe/farm/pull/217))
- support ident pat for tree shaking ([#203](https://github.com/farm-fe/farm/pull/203))
- solve issues when add dependencies in HMR ([#194](https://github.com/farm-fe/farm/pull/194))
- add react antd demo ([#190](https://github.com/farm-fe/farm/pull/190))
- Optimization process error message ([#172](https://github.com/farm-fe/farm/pull/172))
- Refactor Rust plugin system ([#82](https://github.com/farm-fe/farm/pull/82))
- make query part of id ([#85](https://github.com/farm-fe/farm/pull/85))
- optimize cold start speed and fix lazy compilation issue ([#70](https://github.com/farm-fe/farm/pull/70))
- Chore/update template and statistics ([#69](https://github.com/farm-fe/farm/pull/69))
- Feat/static assets ([#61](https://github.com/farm-fe/farm/pull/61))
- Fix/temp config add timestamp ([#46](https://github.com/farm-fe/farm/pull/46))
- write resources to disk to optimize loading time ([#45](https://github.com/farm-fe/farm/pull/45))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
- Feat/hmr ([#26](https://github.com/farm-fe/farm/pull/26))
- implement rust hmr interface ([#25](https://github.com/farm-fe/farm/pull/25))
- Refactor build stage to support HMR ([#24](https://github.com/farm-fe/farm/pull/24))
- Feat/dynamic rust plugin ([#22](https://github.com/farm-fe/farm/pull/22))
- adjust ts core arch
