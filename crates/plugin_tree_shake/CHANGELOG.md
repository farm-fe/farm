# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.15](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.14...farmfe_plugin_tree_shake-v0.0.15) - 2025-08-31

### Added

- Farm v2.0 dev ([#1835](https://github.com/farm-fe/farm/pull/1835))

### Fixed

- tree shake issue when handling cross module top level variables … ([#2166](https://github.com/farm-fe/farm/pull/2166))
- assign reference treeshake ([#2141](https://github.com/farm-fe/farm/pull/2141))

### Other

- support output.asciiOnly ([#2200](https://github.com/farm-fe/farm/pull/2200))
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.14](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.13...farmfe_plugin_tree_shake-v0.0.14) - 2025-01-09

### Other

- updated the following local packages: farmfe_core

## [0.0.13](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.12...farmfe_plugin_tree_shake-v0.0.13) - 2024-12-24

### Fixed

- namespace fallback when use literal computed (#2022)

## [0.0.12](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.11...farmfe_plugin_tree_shake-v0.0.12) - 2024-12-09

### Added

- support tree shake import namespace (#1942)

### Fixed

- bundle export cjs entry file (#1964)
- external alias #1957 (#1959)

## [0.0.11](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.10...farmfe_plugin_tree_shake-v0.0.11) - 2024-10-31

### Other

- simplify string formatting for readability ([#1828](https://github.com/farm-fe/farm/pull/1828))

## [0.0.10](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.9...farmfe_plugin_tree_shake-v0.0.10) - 2024-07-25

### Other
- updated the following local packages: farmfe_core

## [0.0.9](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.8...farmfe_plugin_tree_shake-v0.0.9) - 2024-07-19

### Other
- updated the following local packages: farmfe_core

## [0.0.8](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.7...farmfe_plugin_tree_shake-v0.0.8) - 2024-07-15

### Other
- updated the following local packages: farmfe_core

## [0.0.7](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.6...farmfe_plugin_tree_shake-v0.0.7) - 2024-05-28

### Other
- updated the following local packages: farmfe_core, farmfe_toolkit

## [0.0.6](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.5...farmfe_plugin_tree_shake-v0.0.6) - 2024-05-09

### Other
- update swc to v0.90 ([#1227](https://github.com/farm-fe/farm/pull/1227))

## [0.0.5](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.4...farmfe_plugin_tree_shake-v0.0.5) - 2024-04-13

### Other
- Fix/lazy compile mixed import ([#1175](https://github.com/farm-fe/farm/pull/1175))

## [0.0.4](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.3...farmfe_plugin_tree_shake-v0.0.4) - 2024-04-08

### Added
- improve tree shake traverse ([#1118](https://github.com/farm-fe/farm/pull/1118))

## [0.0.3](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.2...farmfe_plugin_tree_shake-v0.0.3) - 2024-04-01

### Other
- updated the following local packages: farmfe_core, farmfe_testing_helpers

## [0.0.2](https://github.com/farm-fe/farm/compare/farmfe_plugin_tree_shake-v0.0.1...farmfe_plugin_tree_shake-v0.0.2) - 2024-03-24

### Added
- remove if (import.meta.hot) guard for production ([#1030](https://github.com/farm-fe/farm/pull/1030))

### Fixed
- add remove import meta hot condition ([#1056](https://github.com/farm-fe/farm/pull/1056))
- tree shake variable assign ([#1054](https://github.com/farm-fe/farm/pull/1054))
- treeshake class decl assign ([#1038](https://github.com/farm-fe/farm/pull/1038))

### Other
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_plugin_tree_shake-v0.0.1) - 2024-03-12

### Added
- eliminate more useless code ([#971](https://github.com/farm-fe/farm/pull/971))
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- *(hmr)* refactor hmr ([#835](https://github.com/farm-fe/farm/pull/835))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))

### Fixed
- vue bugs ([#973](https://github.com/farm-fe/farm/pull/973))
- [#878](https://github.com/farm-fe/farm/pull/878) ([#881](https://github.com/farm-fe/farm/pull/881))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- css modules sourcemap gen fail ([#621](https://github.com/farm-fe/farm/pull/621))
- module system detection bug ([#345](https://github.com/farm-fe/farm/pull/345))
- tree shake should ignore shake non-script modules ([#210](https://github.com/farm-fe/farm/pull/210))

### Other
- publish crates
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- create farm plugin ([#946](https://github.com/farm-fe/farm/pull/946))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Fix tree-shake self-executed module issue && vite plugin adapterissue ([#626](https://github.com/farm-fe/farm/pull/626))
- *(*)* apply some lint suggestions ([#474](https://github.com/farm-fe/farm/pull/474))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- Optimize tree shake perf ([#369](https://github.com/farm-fe/farm/pull/369))
- solving bugs ([#338](https://github.com/farm-fe/farm/pull/338))
- bugfix/source-module-graph-error ([#192](https://github.com/farm-fe/farm/pull/192))
- support ident pat for tree shaking ([#203](https://github.com/farm-fe/farm/pull/203))
