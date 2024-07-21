# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.10](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.9...farmfe_plugin_runtime-v0.0.10) - 2024-07-19

### Added
- support single library bundle ([#1640](https://github.com/farm-fe/farm/pull/1640))

## [0.0.9](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.8...farmfe_plugin_runtime-v0.0.9) - 2024-07-15

### Other
- updated the following local packages: farmfe_core

## [0.0.8](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.7...farmfe_plugin_runtime-v0.0.8) - 2024-07-11

### Other
- add plugins hooks ([#1581](https://github.com/farm-fe/farm/pull/1581))

## [0.0.7](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.6...farmfe_plugin_runtime-v0.0.7) - 2024-05-28

### Added
- support exclude/include option for html ([#1319](https://github.com/farm-fe/farm/pull/1319))
- support obj external & dts support resolvedPaths ([#1282](https://github.com/farm-fe/farm/pull/1282))

### Fixed
- minify not work ([#1317](https://github.com/farm-fe/farm/pull/1317))
- external inject logic ([#1313](https://github.com/farm-fe/farm/pull/1313))
- cache issues ([#1301](https://github.com/farm-fe/farm/pull/1301))

## [0.0.6](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.5...farmfe_plugin_runtime-v0.0.6) - 2024-05-09

### Added
- support lazy compilation when targeting node ([#1035](https://github.com/farm-fe/farm/pull/1035))
- support top level await ([#1202](https://github.com/farm-fe/farm/pull/1202))

### Fixed
- circle module require ([#1290](https://github.com/farm-fe/farm/pull/1290))
- minify module filter use absolute path ([#1259](https://github.com/farm-fe/farm/pull/1259))
- vite migrations bugs ([#1236](https://github.com/farm-fe/farm/pull/1236))

### Other
- Fix/lazy compilation ([#1253](https://github.com/farm-fe/farm/pull/1253))
- update swc to v0.90 ([#1227](https://github.com/farm-fe/farm/pull/1227))
- add ssr e2e tests ([#1201](https://github.com/farm-fe/farm/pull/1201))

## [0.0.5](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.4...farmfe_plugin_runtime-v0.0.5) - 2024-04-13

### Other
- Fix/lazy compile mixed import ([#1175](https://github.com/farm-fe/farm/pull/1175))

## [0.0.4](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.3...farmfe_plugin_runtime-v0.0.4) - 2024-04-08

### Fixed
- persistent cache conflicts ([#1131](https://github.com/farm-fe/farm/pull/1131))

### Other
- release ([#1115](https://github.com/farm-fe/farm/pull/1115))

## [0.0.3](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.2...farmfe_plugin_runtime-v0.0.3) - 2024-04-01

### Added
- optimize persistent cache ([#1078](https://github.com/farm-fe/farm/pull/1078))

## [0.0.2](https://github.com/farm-fe/farm/compare/farmfe_plugin_runtime-v0.0.1...farmfe_plugin_runtime-v0.0.2) - 2024-03-24

### Added
- minify modules instead of resource pots ([#1025](https://github.com/farm-fe/farm/pull/1025))

### Other
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_plugin_runtime-v0.0.1) - 2024-03-12

### Added
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- *(vite-adapter)* vite plugin unocss compatible ([#853](https://github.com/farm-fe/farm/pull/853))
- Supoort import.meta.url and import.meta.env ([#738](https://github.com/farm-fe/farm/pull/738))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- *(refactor)* RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- support resolve @import and url() dependencies for css ([#367](https://github.com/farm-fe/farm/pull/367))
- support polyfill ([#255](https://github.com/farm-fe/farm/pull/255))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))
- auto external node native module when using farm.config.ts ([#81](https://github.com/farm-fe/farm/pull/81))
- support sourcemap ([#77](https://github.com/farm-fe/farm/pull/77))
- support resolve browser ([#63](https://github.com/farm-fe/farm/pull/63))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- implement the basic compilation flow ([#17](https://github.com/farm-fe/farm/pull/17))

### Fixed
- [#952](https://github.com/farm-fe/farm/pull/952) ([#959](https://github.com/farm-fe/farm/pull/959))
- failed to load external cjs require when output esm ([#892](https://github.com/farm-fe/farm/pull/892))
- [#878](https://github.com/farm-fe/farm/pull/878) ([#881](https://github.com/farm-fe/farm/pull/881))
- [#814](https://github.com/farm-fe/farm/pull/814) ([#816](https://github.com/farm-fe/farm/pull/816))
- [#774](https://github.com/farm-fe/farm/pull/774) ([#777](https://github.com/farm-fe/farm/pull/777))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- issue 747 ([#758](https://github.com/farm-fe/farm/pull/758))
- Isolate runtime from globalThis for script entries ([#446](https://github.com/farm-fe/farm/pull/446))
- css hmr will always reload the whole page ([#413](https://github.com/farm-fe/farm/pull/413))
- module system detection bug ([#345](https://github.com/farm-fe/farm/pull/345))
- windows issue ([#302](https://github.com/farm-fe/farm/pull/302))
- json default ([#270](https://github.com/farm-fe/farm/pull/270))
- lazy compilation and partial bundling bug ([#44](https://github.com/farm-fe/farm/pull/44))

### Other
- publish crates
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- create farm plugin ([#946](https://github.com/farm-fe/farm/pull/946))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- Feat/rollup hook compatible ([#842](https://github.com/farm-fe/farm/pull/842))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- *(*)* apply some lint suggestions ([#474](https://github.com/farm-fe/farm/pull/474))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- add x-data-spreadsheet example ([#422](https://github.com/farm-fe/farm/pull/422))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- update css modules hmr and ci yaml ([#299](https://github.com/farm-fe/farm/pull/299))
- do not resolve browser when target env is node ([#238](https://github.com/farm-fe/farm/pull/238))
- bugfix/source-module-graph-error ([#192](https://github.com/farm-fe/farm/pull/192))
- add profiler and optimize resolve speed ([#217](https://github.com/farm-fe/farm/pull/217))
- add react antd demo ([#190](https://github.com/farm-fe/farm/pull/190))
- Chore/update template and statistics ([#69](https://github.com/farm-fe/farm/pull/69))
- Feat/static assets ([#61](https://github.com/farm-fe/farm/pull/61))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
- Feat/hmr ([#26](https://github.com/farm-fe/farm/pull/26))
- implement rust hmr interface ([#25](https://github.com/farm-fe/farm/pull/25))
- Refactor build stage to support HMR ([#24](https://github.com/farm-fe/farm/pull/24))
- Feat/dynamic rust plugin ([#22](https://github.com/farm-fe/farm/pull/22))
