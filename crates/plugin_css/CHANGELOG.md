# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.16](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.15...farmfe_plugin_css-v0.0.16) - 2024-12-24

### Other

- updated the following local packages: farmfe_core

## [0.0.15](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.14...farmfe_plugin_css-v0.0.15) - 2024-12-09

### Other

- updated the following local packages: farmfe_core

## [0.0.14](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.13...farmfe_plugin_css-v0.0.14) - 2024-10-31

### Added

- *(css)* support css module name coversion ([#1784](https://github.com/farm-fe/farm/pull/1784))

### Fixed

- invalid css [#1748](https://github.com/farm-fe/farm/pull/1748) and [#1557](https://github.com/farm-fe/farm/pull/1557) ([#1823](https://github.com/farm-fe/farm/pull/1823))

### Other

- revert change of replacing invalid css ([#1831](https://github.com/farm-fe/farm/pull/1831))
- simplify string formatting for readability ([#1828](https://github.com/farm-fe/farm/pull/1828))

## [0.0.13](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.12...farmfe_plugin_css-v0.0.13) - 2024-07-25

### Other
- updated the following local packages: farmfe_core

## [0.0.12](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.11...farmfe_plugin_css-v0.0.12) - 2024-07-19

### Fixed
- css @import starting with ~ ([#1622](https://github.com/farm-fe/farm/pull/1622))

## [0.0.11](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.10...farmfe_plugin_css-v0.0.11) - 2024-07-15

### Other
- updated the following local packages: farmfe_core

## [0.0.10](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.9...farmfe_plugin_css-v0.0.10) - 2024-07-11

### Other
- updated the following local packages: farmfe_core, farmfe_utils

## [0.0.9](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.8...farmfe_plugin_css-v0.0.9) - 2024-05-28

### Fixed
- minify not work ([#1317](https://github.com/farm-fe/farm/pull/1317))
- cache issues ([#1301](https://github.com/farm-fe/farm/pull/1301))

## [0.0.8](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.7...farmfe_plugin_css-v0.0.8) - 2024-05-09

### Fixed
- vite migrations bugs ([#1236](https://github.com/farm-fe/farm/pull/1236))
- css import url panic ([#1187](https://github.com/farm-fe/farm/pull/1187))

## [0.0.7](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.6...farmfe_plugin_css-v0.0.7) - 2024-04-13

### Other
- Fix/lazy compile mixed import ([#1175](https://github.com/farm-fe/farm/pull/1175))

## [0.0.6](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.5...farmfe_plugin_css-v0.0.6) - 2024-04-11

### Fixed
- sass import sourcemap ([#1154](https://github.com/farm-fe/farm/pull/1154))

## [0.0.5](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.4...farmfe_plugin_css-v0.0.5) - 2024-04-08

### Fixed
- persistent cache conflicts ([#1131](https://github.com/farm-fe/farm/pull/1131))

## [0.0.4](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.3...farmfe_plugin_css-v0.0.4) - 2024-04-02

### Other
- release ([#1114](https://github.com/farm-fe/farm/pull/1114))

## [0.0.3](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.2...farmfe_plugin_css-v0.0.3) - 2024-04-01

### Fixed
- less/sass url rebase and url publicPath ([#1071](https://github.com/farm-fe/farm/pull/1071))

## [0.0.2](https://github.com/farm-fe/farm/compare/farmfe_plugin_css-v0.0.1...farmfe_plugin_css-v0.0.2) - 2024-03-24

### Added
- minify modules instead of resource pots ([#1025](https://github.com/farm-fe/farm/pull/1025))

### Other
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_plugin_css-v0.0.1) - 2024-03-12

### Added
- eliminate more useless code ([#971](https://github.com/farm-fe/farm/pull/971))
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- *(vite-adapter)* vite plugin unocss compatible ([#853](https://github.com/farm-fe/farm/pull/853))
- *(hmr)* refactor hmr ([#835](https://github.com/farm-fe/farm/pull/835))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- *(refactor)* RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- support write to disk ([#469](https://github.com/farm-fe/farm/pull/469))
- feat support sourcemap chain based on swc sourcemap ([#528](https://github.com/farm-fe/farm/pull/528))
- support resolve @import and url() dependencies for css ([#367](https://github.com/farm-fe/farm/pull/367))
- css module config schema & sourcemap ([#281](https://github.com/farm-fe/farm/pull/281))
- support css modules ([#230](https://github.com/farm-fe/farm/pull/230))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))
- add string when generate css id and change query HashMap to Vec… ([#90](https://github.com/farm-fe/farm/pull/90))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- set up ci for linting and testing ([#4](https://github.com/farm-fe/farm/pull/4))

### Fixed
- vue bugs ([#973](https://github.com/farm-fe/farm/pull/973))
- css url asset ([#827](https://github.com/farm-fe/farm/pull/827))
- [#787](https://github.com/farm-fe/farm/pull/787) [#794](https://github.com/farm-fe/farm/pull/794) [#785](https://github.com/farm-fe/farm/pull/785) ([#800](https://github.com/farm-fe/farm/pull/800))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- [#693](https://github.com/farm-fe/farm/pull/693) ([#695](https://github.com/farm-fe/farm/pull/695))
- bugs when migrate from vite to farm ([#665](https://github.com/farm-fe/farm/pull/665))
- css modules sourcemap gen fail ([#621](https://github.com/farm-fe/farm/pull/621))
- lazy compialtion error and windows css error ([#454](https://github.com/farm-fe/farm/pull/454))
- Isolate runtime from globalThis for script entries ([#446](https://github.com/farm-fe/farm/pull/446))
- css import not work ([#437](https://github.com/farm-fe/farm/pull/437))
- css hmr will always reload the whole page ([#413](https://github.com/farm-fe/farm/pull/413))
- css dev bugs and add antd vue examples ([#320](https://github.com/farm-fe/farm/pull/320))
- windows issue ([#302](https://github.com/farm-fe/farm/pull/302))
- hmr does not remove css ([#103](https://github.com/farm-fe/farm/pull/103))

### Other
- publish crates
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- Feat/rollup hook compatible ([#842](https://github.com/farm-fe/farm/pull/842))
- *(persistent-cache)* optimize cache ([#782](https://github.com/farm-fe/farm/pull/782))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Fix/js plugins filters ([#678](https://github.com/farm-fe/farm/pull/678))
- Fix tree-shake self-executed module issue && vite plugin adapterissue ([#626](https://github.com/farm-fe/farm/pull/626))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Feat/js plugin adaptor ([#613](https://github.com/farm-fe/farm/pull/613))
- *(*)* apply some lint suggestions ([#474](https://github.com/farm-fe/farm/pull/474))
- add more features ([#387](https://github.com/farm-fe/farm/pull/387))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- Feat/resource module order ([#312](https://github.com/farm-fe/farm/pull/312))
- make the modules order be execution order ([#311](https://github.com/farm-fe/farm/pull/311))
- update css modules hmr and ci yaml ([#299](https://github.com/farm-fe/farm/pull/299))
- Feat/css prefixer ([#294](https://github.com/farm-fe/farm/pull/294))
- Refactor Rust plugin system ([#82](https://github.com/farm-fe/farm/pull/82))
- Fix/temp config add timestamp ([#46](https://github.com/farm-fe/farm/pull/46))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
