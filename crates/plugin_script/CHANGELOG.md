# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.2](https://github.com/farm-fe/farm/compare/farmfe_plugin_script-v0.0.1...farmfe_plugin_script-v0.0.2) - 2024-03-24

### Added
- minify modules instead of resource pots ([#1025](https://github.com/farm-fe/farm/pull/1025))

### Fixed
- vite project migration issues ([#1060](https://github.com/farm-fe/farm/pull/1060))

### Other
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_plugin_script-v0.0.1) - 2024-03-12

### Added
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- *(hmr)* refactor hmr ([#835](https://github.com/farm-fe/farm/pull/835))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- support import.meta.glob ([#658](https://github.com/farm-fe/farm/pull/658))
- *(refactor)* RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- feat support sourcemap chain based on swc sourcemap ([#528](https://github.com/farm-fe/farm/pull/528))
- remove chokidar and add a rust file watcher ([#471](https://github.com/farm-fe/farm/pull/471))
- support resolve @import and url() dependencies for css ([#367](https://github.com/farm-fe/farm/pull/367))
- support swc plugins ([#199](https://github.com/farm-fe/farm/pull/199))
- css module config schema & sourcemap ([#281](https://github.com/farm-fe/farm/pull/281))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- feat entry key as resource name ([#205](https://github.com/farm-fe/farm/pull/205))
- add string when generate css id and change query HashMap to Vec… ([#90](https://github.com/farm-fe/farm/pull/90))
- support sourcemap ([#77](https://github.com/farm-fe/farm/pull/77))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- implement the basic compilation flow ([#17](https://github.com/farm-fe/farm/pull/17))
- init project with cargo and pnpm

### Fixed
- win32-ia32 ci build ([#826](https://github.com/farm-fe/farm/pull/826))
- [#814](https://github.com/farm-fe/farm/pull/814) ([#816](https://github.com/farm-fe/farm/pull/816))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- issue 747 ([#758](https://github.com/farm-fe/farm/pull/758))
- runtime resource panic ([#749](https://github.com/farm-fe/farm/pull/749))
- bugs ([#710](https://github.com/farm-fe/farm/pull/710))
- issue [#652](https://github.com/farm-fe/farm/pull/652) ([#655](https://github.com/farm-fe/farm/pull/655))
- create farm error ([#630](https://github.com/farm-fe/farm/pull/630))
- lazy compialtion error and windows css error ([#454](https://github.com/farm-fe/farm/pull/454))
- Isolate runtime from globalThis for script entries ([#446](https://github.com/farm-fe/farm/pull/446))
- lazy compilation and deadlock ([#370](https://github.com/farm-fe/farm/pull/370))
- vue migrate bugs ([#357](https://github.com/farm-fe/farm/pull/357))
- module system detection bug ([#345](https://github.com/farm-fe/farm/pull/345))
- lazy compilation and partial bundling bug ([#44](https://github.com/farm-fe/farm/pull/44))

### Other
- publish crates
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- Fix transformIndexHtml does not work as expected ([#884](https://github.com/farm-fe/farm/pull/884))
- Feat/rollup hook compatible ([#842](https://github.com/farm-fe/farm/pull/842))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Fix/js plugins filters ([#678](https://github.com/farm-fe/farm/pull/678))
- normalzie js plugins options ([#668](https://github.com/farm-fe/farm/pull/668))
- Fix tree-shake self-executed module issue && vite plugin adapterissue ([#626](https://github.com/farm-fe/farm/pull/626))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Feat/js plugin adaptor ([#613](https://github.com/farm-fe/farm/pull/613))
- update swc and support emotion ([#500](https://github.com/farm-fe/farm/pull/500))
- *(*)* apply some lint suggestions ([#474](https://github.com/farm-fe/farm/pull/474))
- support postcss ([#455](https://github.com/farm-fe/farm/pull/455))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- pretty syntax error ([#372](https://github.com/farm-fe/farm/pull/372))
- solving bugs ([#338](https://github.com/farm-fe/farm/pull/338))
- update css modules hmr and ci yaml ([#299](https://github.com/farm-fe/farm/pull/299))
- format with prettier ([#266](https://github.com/farm-fe/farm/pull/266))
- do not resolve browser when target env is node ([#238](https://github.com/farm-fe/farm/pull/238))
- support ident pat for tree shaking ([#203](https://github.com/farm-fe/farm/pull/203))
- Refactor Rust plugin system ([#82](https://github.com/farm-fe/farm/pull/82))
- make query part of id ([#85](https://github.com/farm-fe/farm/pull/85))
- Chore/update template and statistics ([#69](https://github.com/farm-fe/farm/pull/69))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
- Feat/hmr ([#26](https://github.com/farm-fe/farm/pull/26))
- implement rust hmr interface ([#25](https://github.com/farm-fe/farm/pull/25))
- Refactor build stage to support HMR ([#24](https://github.com/farm-fe/farm/pull/24))
- Feat/dynamic rust plugin ([#22](https://github.com/farm-fe/farm/pull/22))
