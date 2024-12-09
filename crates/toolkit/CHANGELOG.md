# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.15](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.14...farmfe_toolkit-v0.0.15) - 2024-12-09

### Other

- updated the following local packages: farmfe_core

## [0.0.14](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.13...farmfe_toolkit-v0.0.14) - 2024-10-31

### Fixed

- plugin circle call & dynamic import ([#1901](https://github.com/farm-fe/farm/pull/1901))
- css resolving issue ([#1834](https://github.com/farm-fe/farm/pull/1834))
- invalid css [#1748](https://github.com/farm-fe/farm/pull/1748) and [#1557](https://github.com/farm-fe/farm/pull/1557) ([#1823](https://github.com/farm-fe/farm/pull/1823))
- arcgis lazy compile fail ([#1750](https://github.com/farm-fe/farm/pull/1750))
- ignore non-utf8 error when getting file contents ([#1799](https://github.com/farm-fe/farm/pull/1799))
- preset_env plugin match and module system `analyze` ([#1751](https://github.com/farm-fe/farm/pull/1751))
- globalThis undefined ([#1726](https://github.com/farm-fe/farm/pull/1726))
- [#1672](https://github.com/farm-fe/farm/pull/1672) ([#1692](https://github.com/farm-fe/farm/pull/1692))

### Other

- revert change of replacing invalid css ([#1831](https://github.com/farm-fe/farm/pull/1831))
- simplify string formatting for readability ([#1828](https://github.com/farm-fe/farm/pull/1828))

## [0.0.13](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.12...farmfe_toolkit-v0.0.13) - 2024-07-25

### Other
- updated the following local packages: farmfe_core

## [0.0.12](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.11...farmfe_toolkit-v0.0.12) - 2024-07-19

### Other
- updated the following local packages: farmfe_core

## [0.0.11](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.10...farmfe_toolkit-v0.0.11) - 2024-07-15

### Other
- updated the following local packages: farmfe_core

## [0.0.10](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.9...farmfe_toolkit-v0.0.10) - 2024-07-11

### Other
- updated the following local packages: farmfe_core, farmfe_utils

## [0.0.9](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.8...farmfe_toolkit-v0.0.9) - 2024-05-28

### Added
- support exclude/include option for html ([#1319](https://github.com/farm-fe/farm/pull/1319))

### Fixed
- cache issues ([#1301](https://github.com/farm-fe/farm/pull/1301))

## [0.0.8](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.7...farmfe_toolkit-v0.0.8) - 2024-05-09

### Added
- support lazy compilation when targeting node ([#1035](https://github.com/farm-fe/farm/pull/1035))

### Other
- Fix/lazy compilation ([#1253](https://github.com/farm-fe/farm/pull/1253))
- update swc to v0.90 ([#1227](https://github.com/farm-fe/farm/pull/1227))

## [0.0.7](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.6...farmfe_toolkit-v0.0.7) - 2024-04-13

### Other
- updated the following local packages: farmfe_core

## [0.0.6](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.5...farmfe_toolkit-v0.0.6) - 2024-04-08

### Other
- less strict html parsing ([#1138](https://github.com/farm-fe/farm/pull/1138))

## [0.0.5](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.4...farmfe_toolkit-v0.0.5) - 2024-04-02

### Other
- release ([#1114](https://github.com/farm-fe/farm/pull/1114))

## [0.0.4](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.3...farmfe_toolkit-v0.0.4) - 2024-04-01

### Other
- updated the following local packages: farmfe_core, farmfe_testing_helpers

## [0.0.3](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.2...farmfe_toolkit-v0.0.3) - 2024-03-24

### Added
- minify modules instead of resource pots ([#1025](https://github.com/farm-fe/farm/pull/1025))

### Fixed
- vite project migration issues ([#1060](https://github.com/farm-fe/farm/pull/1060))

### Other
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.0.2](https://github.com/farm-fe/farm/compare/farmfe_toolkit-v0.0.1...farmfe_toolkit-v0.0.2) - 2024-03-13

### Other
- release ([#1009](https://github.com/farm-fe/farm/pull/1009))

## [0.0.1](https://github.com/farm-fe/farm/releases/tag/farmfe_toolkit-v0.0.1) - 2024-03-12

### Added
- eliminate more useless code ([#971](https://github.com/farm-fe/farm/pull/971))
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- *(refactor)* RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- feat support sourcemap chain based on swc sourcemap ([#528](https://github.com/farm-fe/farm/pull/528))
- support swc plugins ([#199](https://github.com/farm-fe/farm/pull/199))
- css module config schema & sourcemap ([#281](https://github.com/farm-fe/farm/pull/281))
- support polyfill ([#255](https://github.com/farm-fe/farm/pull/255))
- support css modules ([#230](https://github.com/farm-fe/farm/pull/230))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))
- support sourcemap ([#77](https://github.com/farm-fe/farm/pull/77))
- support resolve browser ([#63](https://github.com/farm-fe/farm/pull/63))
- serve resources with dev server ([#21](https://github.com/farm-fe/farm/pull/21))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- implement the basic compilation flow ([#17](https://github.com/farm-fe/farm/pull/17))

### Fixed
- [#952](https://github.com/farm-fe/farm/pull/952) ([#959](https://github.com/farm-fe/farm/pull/959))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- issue 747 ([#758](https://github.com/farm-fe/farm/pull/758))
- mode set to development when build ([#439](https://github.com/farm-fe/farm/pull/439))
- vue migrate bugs ([#357](https://github.com/farm-fe/farm/pull/357))
- module system detection bug ([#345](https://github.com/farm-fe/farm/pull/345))

### Other
- publish crates
- update rust plugin scripts
- bump 1.0.0-beta ([#1011](https://github.com/farm-fe/farm/pull/1011))
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))
- [#997](https://github.com/farm-fe/farm/pull/997) ([#1003](https://github.com/farm-fe/farm/pull/1003))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- Fix/bugs ([#640](https://github.com/farm-fe/farm/pull/640))
- update swc and support emotion ([#500](https://github.com/farm-fe/farm/pull/500))
- *(*)* apply some lint suggestions ([#474](https://github.com/farm-fe/farm/pull/474))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- pretty syntax error ([#372](https://github.com/farm-fe/farm/pull/372))
- update css modules hmr and ci yaml ([#299](https://github.com/farm-fe/farm/pull/299))
- Feat/css prefixer ([#294](https://github.com/farm-fe/farm/pull/294))
- format with prettier ([#266](https://github.com/farm-fe/farm/pull/266))
- add profiler and optimize resolve speed ([#217](https://github.com/farm-fe/farm/pull/217))
- support ident pat for tree shaking ([#203](https://github.com/farm-fe/farm/pull/203))
- Refactor Rust plugin system ([#82](https://github.com/farm-fe/farm/pull/82))
- Chore/update template and statistics ([#69](https://github.com/farm-fe/farm/pull/69))
- Feat/static assets ([#61](https://github.com/farm-fe/farm/pull/61))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
- Feat/hmr ([#26](https://github.com/farm-fe/farm/pull/26))
- Refactor build stage to support HMR ([#24](https://github.com/farm-fe/farm/pull/24))
- Feat/dynamic rust plugin ([#22](https://github.com/farm-fe/farm/pull/22))
