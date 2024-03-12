# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
