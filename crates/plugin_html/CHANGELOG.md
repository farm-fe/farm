# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.12](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.11...farmfe_plugin_html-v0.0.12) - 2024-07-29

### Other
- updated the following local packages: farmfe_core

## [0.0.11](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.10...farmfe_plugin_html-v0.0.11) - 2024-07-25

### Other
- updated the following local packages: farmfe_core

## [0.0.10](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.9...farmfe_plugin_html-v0.0.10) - 2024-07-19

### Fixed
- public path undefine when normalizing hmr path ([#1638](https://github.com/farm-fe/farm/pull/1638))

## [0.0.9](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.8...farmfe_plugin_html-v0.0.9) - 2024-07-15

### Other
- updated the following local packages: farmfe_core

## [0.0.8](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.7...farmfe_plugin_html-v0.0.8) - 2024-07-11

### Other
- updated the following local packages: farmfe_core

## [0.0.7](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.6...farmfe_plugin_html-v0.0.7) - 2024-05-28

### Added
- support exclude/include option for html ([#1319](https://github.com/farm-fe/farm/pull/1319))

### Fixed
- cache issues ([#1301](https://github.com/farm-fe/farm/pull/1301))

## [0.0.6](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.5...farmfe_plugin_html-v0.0.6) - 2024-05-09

### Added
- support isolate runtime resource ([#1200](https://github.com/farm-fe/farm/pull/1200))

### Fixed
- vite migrations bugs ([#1236](https://github.com/farm-fe/farm/pull/1236))

### Other
- Fix/lazy compilation ([#1253](https://github.com/farm-fe/farm/pull/1253))

## [0.0.5](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.4...farmfe_plugin_html-v0.0.5) - 2024-04-13

### Other
- Fix/lazy compile mixed import ([#1175](https://github.com/farm-fe/farm/pull/1175))

## [0.0.4](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.3...farmfe_plugin_html-v0.0.4) - 2024-04-08

### Fixed
- make publicPath and publicDir configuration work properly. ([#1121](https://github.com/farm-fe/farm/pull/1121))

### Other
- release ([#1115](https://github.com/farm-fe/farm/pull/1115))
- release ([#1114](https://github.com/farm-fe/farm/pull/1114))

## [0.0.3](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.2...farmfe_plugin_html-v0.0.3) - 2024-04-01

### Other
- updated the following local packages: farmfe_core, farmfe_testing_helpers

## [0.0.2](https://github.com/farm-fe/farm/compare/farmfe_plugin_html-v0.0.1...farmfe_plugin_html-v0.0.2) - 2024-03-24

### Added
- minify modules instead of resource pots ([#1025](https://github.com/farm-fe/farm/pull/1025))

### Fixed
- vite project migration issues ([#1060](https://github.com/farm-fe/farm/pull/1060))

### Other
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_plugin_html-v0.0.1) - 2024-03-12

### Added
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- *(refactor)* RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- support resolve @import and url() dependencies for css ([#367](https://github.com/farm-fe/farm/pull/367))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))
- add string when generate css id and change query HashMap to Vec… ([#90](https://github.com/farm-fe/farm/pull/90))
- auto external node native module when using farm.config.ts ([#81](https://github.com/farm-fe/farm/pull/81))
- support sourcemap ([#77](https://github.com/farm-fe/farm/pull/77))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- set up ci for linting and testing ([#4](https://github.com/farm-fe/farm/pull/4))

### Fixed
- [#787](https://github.com/farm-fe/farm/pull/787) [#794](https://github.com/farm-fe/farm/pull/794) [#785](https://github.com/farm-fe/farm/pull/785) ([#800](https://github.com/farm-fe/farm/pull/800))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- public path issue for css and static assets ([#561](https://github.com/farm-fe/farm/pull/561))
- lazy compialtion error and windows css error ([#454](https://github.com/farm-fe/farm/pull/454))
- Isolate runtime from globalThis for script entries ([#446](https://github.com/farm-fe/farm/pull/446))
- vue migrate bugs ([#357](https://github.com/farm-fe/farm/pull/357))
- lazy compilation and partial bundling bug ([#44](https://github.com/farm-fe/farm/pull/44))

### Other
- publish crates
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- support absolute path for script tag ([#927](https://github.com/farm-fe/farm/pull/927))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- Fix transformIndexHtml does not work as expected ([#884](https://github.com/farm-fe/farm/pull/884))
- Feat/rollup hook compatible ([#842](https://github.com/farm-fe/farm/pull/842))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Feat/js plugin adaptor ([#613](https://github.com/farm-fe/farm/pull/613))
- *(*)* apply some lint suggestions ([#474](https://github.com/farm-fe/farm/pull/474))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- make resources order injected to html execution order ([#448](https://github.com/farm-fe/farm/pull/448))
- add more features ([#387](https://github.com/farm-fe/farm/pull/387))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- solving bugs ([#338](https://github.com/farm-fe/farm/pull/338))
- bugfix
- Refactor Rust plugin system ([#82](https://github.com/farm-fe/farm/pull/82))
- make query part of id ([#85](https://github.com/farm-fe/farm/pull/85))
- Feat/static assets ([#61](https://github.com/farm-fe/farm/pull/61))
- Fix/temp config add timestamp ([#46](https://github.com/farm-fe/farm/pull/46))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
- Feat/hmr ([#26](https://github.com/farm-fe/farm/pull/26))
- implement rust hmr interface ([#25](https://github.com/farm-fe/farm/pull/25))
- Feat/dynamic rust plugin ([#22](https://github.com/farm-fe/farm/pull/22))
