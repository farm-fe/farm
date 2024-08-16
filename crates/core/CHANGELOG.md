# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.5](https://github.com/farm-fe/farm/compare/farmfe_core-v0.6.4...farmfe_core-v0.6.5) - 2024-08-16

### Fixed
- single bundle ([#1653](https://github.com/farm-fe/farm/pull/1653))

## [0.6.4](https://github.com/farm-fe/farm/compare/farmfe_core-v0.6.3...farmfe_core-v0.6.4) - 2024-07-25

### Added
- support targetEnv library library-browser/node ([#1656](https://github.com/farm-fe/farm/pull/1656))

## [0.6.3](https://github.com/farm-fe/farm/compare/farmfe_core-v0.6.2...farmfe_core-v0.6.3) - 2024-07-19

### Fixed
- import/export minify duplicate identifier [#1625](https://github.com/farm-fe/farm/pull/1625) ([#1634](https://github.com/farm-fe/farm/pull/1634))

## [0.6.2](https://github.com/farm-fe/farm/compare/farmfe_core-v0.6.1...farmfe_core-v0.6.2) - 2024-07-15

### Other
- update crates deps ([#1611](https://github.com/farm-fe/farm/pull/1611))

## [0.6.1](https://github.com/farm-fe/farm/compare/farmfe_core-v0.6.0...farmfe_core-v0.6.1) - 2024-07-11

### Fixed
- disable swc remove import, fix [#1555](https://github.com/farm-fe/farm/pull/1555) ([#1565](https://github.com/farm-fe/farm/pull/1565))

### Other
- simple performance optimize ([#1566](https://github.com/farm-fe/farm/pull/1566))
- add plugins hooks ([#1581](https://github.com/farm-fe/farm/pull/1581))

## [0.6.0](https://github.com/farm-fe/farm/compare/farmfe_core-v0.5.1...farmfe_core-v0.6.0) - 2024-05-28

### Added
- support exclude/include option for html ([#1319](https://github.com/farm-fe/farm/pull/1319))
- support obj external & dts support resolvedPaths ([#1282](https://github.com/farm-fe/farm/pull/1282))

### Fixed
- cache issues ([#1301](https://github.com/farm-fe/farm/pull/1301))

### Other
- optimize record manager ([#1303](https://github.com/farm-fe/farm/pull/1303))

## [0.5.1](https://github.com/farm-fe/farm/compare/farmfe_core-v0.5.0...farmfe_core-v0.5.1) - 2024-05-09

### Added
- support lazy compilation when targeting node ([#1035](https://github.com/farm-fe/farm/pull/1035))

### Other
- Fix/lazy compilation ([#1253](https://github.com/farm-fe/farm/pull/1253))
- Version Packages v1.1.0 ([#1214](https://github.com/farm-fe/farm/pull/1214))
- update swc to v0.90 ([#1227](https://github.com/farm-fe/farm/pull/1227))

## [0.5.0](https://github.com/farm-fe/farm/compare/farmfe_core-v0.4.5...farmfe_core-v0.5.0) - 2024-04-13

### Other
- Fix/lazy compile mixed import ([#1175](https://github.com/farm-fe/farm/pull/1175))

## [0.4.5](https://github.com/farm-fe/farm/compare/farmfe_core-v0.4.4...farmfe_core-v0.4.5) - 2024-04-08

### Fixed
- persistent cache conflicts ([#1131](https://github.com/farm-fe/farm/pull/1131))

### Other
- Chore/example vue2 ([#1127](https://github.com/farm-fe/farm/pull/1127))

## [0.4.4](https://github.com/farm-fe/farm/compare/farmfe_core-v0.4.3...farmfe_core-v0.4.4) - 2024-04-02

### Other
- lock swc version ([#1113](https://github.com/farm-fe/farm/pull/1113))

## [0.4.3](https://github.com/farm-fe/farm/compare/farmfe_core-v0.4.2...farmfe_core-v0.4.3) - 2024-04-01

### Fixed
- less/sass url rebase and url publicPath ([#1071](https://github.com/farm-fe/farm/pull/1071))

### Other
- *(splitQuery)* change split to splitOnce ([#1068](https://github.com/farm-fe/farm/pull/1068))

## [0.4.2](https://github.com/farm-fe/farm/compare/farmfe_core-v0.4.1...farmfe_core-v0.4.2) - 2024-03-24

### Added
- farm e2e test ([#1041](https://github.com/farm-fe/farm/pull/1041)) ([#1049](https://github.com/farm-fe/farm/pull/1049))
- minify modules instead of resource pots ([#1025](https://github.com/farm-fe/farm/pull/1025))

### Fixed
- vite project migration issues ([#1060](https://github.com/farm-fe/farm/pull/1060))

### Other
- support glob brace ([#1055](https://github.com/farm-fe/farm/pull/1055))
- Feat/update readme ([#1028](https://github.com/farm-fe/farm/pull/1028))

## [0.4.1](https://github.com/farm-fe/farm/compare/farmfe_core-v0.4.0...farmfe_core-v0.4.1) - 2024-03-13

### Other
- updated the following local packages: farmfe_utils

## [0.4.0](https://github.com/farm-fe/farm/compare/farmfe_core-v0.3.0...farmfe_core-v0.4.0) - 2024-03-12

### Other
- ready to release 1.0.0-beta ([#936](https://github.com/farm-fe/farm/pull/936))

## [0.3.0](https://github.com/farm-fe/farm/compare/farmfe_core-v0.2.4...farmfe_core-v0.3.0) - 2024-03-08

### Other
- [#997](https://github.com/farm-fe/farm/pull/997) ([#1003](https://github.com/farm-fe/farm/pull/1003))

## [0.2.4](https://github.com/farm-fe/farm/compare/farmfe_core-v0.2.3...farmfe_core-v0.2.4) - 2024-02-12

### Fixed
- copy artifacts ([#978](https://github.com/farm-fe/farm/pull/978))

## [0.2.3](https://github.com/farm-fe/farm/compare/farmfe_core-v0.2.2...farmfe_core-v0.2.3) - 2024-02-07

### Fixed
- vue bugs ([#973](https://github.com/farm-fe/farm/pull/973))

## [0.2.2](https://github.com/farm-fe/farm/compare/farmfe_core-v0.2.1...farmfe_core-v0.2.2) - 2024-02-06

### Added
- Provide universal JavaScript Api ([#944](https://github.com/farm-fe/farm/pull/944))
- add progress plugin ([#948](https://github.com/farm-fe/farm/pull/948))
- add filter for augmentResourceHash & renderResourcePot hook ([#899](https://github.com/farm-fe/farm/pull/899))
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- add overlay window and optimize socket client interaction code ([#854](https://github.com/farm-fe/farm/pull/854))
- *(hmr)* refactor hmr ([#835](https://github.com/farm-fe/farm/pull/835))
- vue & solid ssr examples ([#849](https://github.com/farm-fe/farm/pull/849))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- js plugin record viewer ([#661](https://github.com/farm-fe/farm/pull/661))
- *(refactor)* RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- Support js plugin hook context methods for unplugin ([#589](https://github.com/farm-fe/farm/pull/589))
- generate stage add record ([#573](https://github.com/farm-fe/farm/pull/573))
- feat support sourcemap chain based on swc sourcemap ([#528](https://github.com/farm-fe/farm/pull/528))
- add record manager ([#511](https://github.com/farm-fe/farm/pull/511))
- support add extra watch file ([#470](https://github.com/farm-fe/farm/pull/470))
- support resolve @import and url() dependencies for css ([#367](https://github.com/farm-fe/farm/pull/367))
- support swc plugins ([#199](https://github.com/farm-fe/farm/pull/199))
- css module config schema & sourcemap ([#281](https://github.com/farm-fe/farm/pull/281))
- support polyfill ([#255](https://github.com/farm-fe/farm/pull/255))
- support css modules ([#230](https://github.com/farm-fe/farm/pull/230))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- *(resolve)* add `TargetEnv` used to judgementthe current output environment ([#216](https://github.com/farm-fe/farm/pull/216))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))
- plugin resolve exports ([#150](https://github.com/farm-fe/farm/pull/150))
- add string when generate css id and change query HashMap to Vec… ([#90](https://github.com/farm-fe/farm/pull/90))
- support sourcemap ([#77](https://github.com/farm-fe/farm/pull/77))
- support resolve browser ([#63](https://github.com/farm-fe/farm/pull/63))
- add load and transform hook for js plugins ([#58](https://github.com/farm-fe/farm/pull/58))
- serve resources with dev server ([#21](https://github.com/farm-fe/farm/pull/21))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- implement the basic compilation flow ([#17](https://github.com/farm-fe/farm/pull/17))
- setup node binding and tests ([#8](https://github.com/farm-fe/farm/pull/8))
- set up ci for linting and testing ([#4](https://github.com/farm-fe/farm/pull/4))
- init project with cargo and pnpm

### Fixed
- [#857](https://github.com/farm-fe/farm/pull/857) [#460](https://github.com/farm-fe/farm/pull/460) ([#896](https://github.com/farm-fe/farm/pull/896))
- [#878](https://github.com/farm-fe/farm/pull/878) ([#881](https://github.com/farm-fe/farm/pull/881))
- [#850](https://github.com/farm-fe/farm/pull/850) ([#870](https://github.com/farm-fe/farm/pull/870))
- [#814](https://github.com/farm-fe/farm/pull/814) ([#816](https://github.com/farm-fe/farm/pull/816))
- [#770](https://github.com/farm-fe/farm/pull/770) ([#807](https://github.com/farm-fe/farm/pull/807))
- [#787](https://github.com/farm-fe/farm/pull/787) [#794](https://github.com/farm-fe/farm/pull/794) [#785](https://github.com/farm-fe/farm/pull/785) ([#800](https://github.com/farm-fe/farm/pull/800))
- immutable module not found ([#788](https://github.com/farm-fe/farm/pull/788))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- [#768](https://github.com/farm-fe/farm/pull/768) ([#771](https://github.com/farm-fe/farm/pull/771))
- fix [#760](https://github.com/farm-fe/farm/pull/760) [#761](https://github.com/farm-fe/farm/pull/761) ([#765](https://github.com/farm-fe/farm/pull/765))
- issue 747 ([#758](https://github.com/farm-fe/farm/pull/758))
- bugs ([#710](https://github.com/farm-fe/farm/pull/710))
- config record not work in transform hook ([#707](https://github.com/farm-fe/farm/pull/707))
- [#693](https://github.com/farm-fe/farm/pull/693) ([#695](https://github.com/farm-fe/farm/pull/695))
- bugs when migrate from vite to farm ([#665](https://github.com/farm-fe/farm/pull/665))
- Error in export field lookup algorithm ([#635](https://github.com/farm-fe/farm/pull/635))
- css modules sourcemap gen fail ([#621](https://github.com/farm-fe/farm/pull/621))
- lazy compialtion error and windows css error ([#454](https://github.com/farm-fe/farm/pull/454))
- css hmr will always reload the whole page ([#413](https://github.com/farm-fe/farm/pull/413))
- vue migrate bugs ([#357](https://github.com/farm-fe/farm/pull/357))
- module system detection bug ([#345](https://github.com/farm-fe/farm/pull/345))
- add default config for script ([#246](https://github.com/farm-fe/farm/pull/246))
- js plugin cannot use custom type ([#168](https://github.com/farm-fe/farm/pull/168))
- lazy compilation and partial bundling bug ([#44](https://github.com/farm-fe/farm/pull/44))

### Other
- create farm plugin ([#946](https://github.com/farm-fe/farm/pull/946))
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- switch to chokidar ([#886](https://github.com/farm-fe/farm/pull/886))
- Feat/rollup hook compatible ([#842](https://github.com/farm-fe/farm/pull/842))
- optimize cache 4 ([#820](https://github.com/farm-fe/farm/pull/820))
- optimize cache 3 ([#786](https://github.com/farm-fe/farm/pull/786))
- optimize cache 2 ([#785](https://github.com/farm-fe/farm/pull/785))
- *(persistent-cache)* optimize cache ([#782](https://github.com/farm-fe/farm/pull/782))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- update record viewer ([#731](https://github.com/farm-fe/farm/pull/731))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- record  add hmr flag ([#696](https://github.com/farm-fe/farm/pull/696))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Feat/js plugin adaptor ([#613](https://github.com/farm-fe/farm/pull/613))
- support buildStart, buildEnd, updateModules js plugins hook ([#574](https://github.com/farm-fe/farm/pull/574))
- version bump ([#563](https://github.com/farm-fe/farm/pull/563))
- js plugin support finish hook ([#513](https://github.com/farm-fe/farm/pull/513))
- update swc and support emotion ([#500](https://github.com/farm-fe/farm/pull/500))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- add more features ([#387](https://github.com/farm-fe/farm/pull/387))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- pretty syntax error ([#372](https://github.com/farm-fe/farm/pull/372))
- Optimize tree shake perf ([#369](https://github.com/farm-fe/farm/pull/369))
- solving bugs ([#338](https://github.com/farm-fe/farm/pull/338))
- make the modules order be execution order ([#311](https://github.com/farm-fe/farm/pull/311))
- update css modules hmr and ci yaml ([#299](https://github.com/farm-fe/farm/pull/299))
- Feat/css prefixer ([#294](https://github.com/farm-fe/farm/pull/294))
- do not resolve browser when target env is node ([#238](https://github.com/farm-fe/farm/pull/238))
- bugfix/source-module-graph-error ([#192](https://github.com/farm-fe/farm/pull/192))
- add profiler and optimize resolve speed ([#217](https://github.com/farm-fe/farm/pull/217))
- support ident pat for tree shaking ([#203](https://github.com/farm-fe/farm/pull/203))
- solve issues when add dependencies in HMR ([#194](https://github.com/farm-fe/farm/pull/194))
- Refactor Rust plugin system ([#82](https://github.com/farm-fe/farm/pull/82))
- make query part of id ([#85](https://github.com/farm-fe/farm/pull/85))
- Feat/static assets ([#61](https://github.com/farm-fe/farm/pull/61))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
- Feat/hmr ([#26](https://github.com/farm-fe/farm/pull/26))
- implement rust hmr interface ([#25](https://github.com/farm-fe/farm/pull/25))
- Refactor build stage to support HMR ([#24](https://github.com/farm-fe/farm/pull/24))
- Feat/dynamic rust plugin ([#22](https://github.com/farm-fe/farm/pull/22))
- adjust ts core arch

## [0.2.1](https://github.com/farm-fe/farm/compare/farmfe_core-v0.2.0...farmfe_core-v0.2.1) - 2024-02-06

### Added

- Provide universal JavaScript Api ([#944](https://github.com/farm-fe/farm/pull/944))
- add progress plugin ([#948](https://github.com/farm-fe/farm/pull/948))
- add filter for augmentResourceHash & renderResourcePot hook ([#899](https://github.com/farm-fe/farm/pull/899))
- preserve comments [#607](https://github.com/farm-fe/farm/pull/607) ([#900](https://github.com/farm-fe/farm/pull/900))
- add overlay window and optimize socket client interaction code ([#854](https://github.com/farm-fe/farm/pull/854))
- _(hmr)_ refactor hmr ([#835](https://github.com/farm-fe/farm/pull/835))
- vue & solid ssr examples ([#849](https://github.com/farm-fe/farm/pull/849))
- Support persistent cache and incremental building ([#476](https://github.com/farm-fe/farm/pull/476))
- js plugin record viewer ([#661](https://github.com/farm-fe/farm/pull/661))
- _(refactor)_ RFC-003 New Partial Bundling Algorithm ([#559](https://github.com/farm-fe/farm/pull/559))
- Support js plugin hook context methods for unplugin ([#589](https://github.com/farm-fe/farm/pull/589))
- generate stage add record ([#573](https://github.com/farm-fe/farm/pull/573))
- feat support sourcemap chain based on swc sourcemap ([#528](https://github.com/farm-fe/farm/pull/528))
- add record manager ([#511](https://github.com/farm-fe/farm/pull/511))
- support add extra watch file ([#470](https://github.com/farm-fe/farm/pull/470))
- support resolve @import and url() dependencies for css ([#367](https://github.com/farm-fe/farm/pull/367))
- support swc plugins ([#199](https://github.com/farm-fe/farm/pull/199))
- css module config schema & sourcemap ([#281](https://github.com/farm-fe/farm/pull/281))
- support polyfill ([#255](https://github.com/farm-fe/farm/pull/255))
- support css modules ([#230](https://github.com/farm-fe/farm/pull/230))
- support script minification ([#191](https://github.com/farm-fe/farm/pull/191))
- _(resolve)_ add `TargetEnv` used to judgement the current output environment ([#216](https://github.com/farm-fe/farm/pull/216))
- tree shake ([#99](https://github.com/farm-fe/farm/pull/99))
- plugin resolve exports ([#150](https://github.com/farm-fe/farm/pull/150))
- add string when generate css id and change query HashMap to Vec… ([#90](https://github.com/farm-fe/farm/pull/90))
- support sourcemap ([#77](https://github.com/farm-fe/farm/pull/77))
- support resolve browser ([#63](https://github.com/farm-fe/farm/pull/63))
- add load and transform hook for js plugins ([#58](https://github.com/farm-fe/farm/pull/58))
- serve resources with dev server ([#21](https://github.com/farm-fe/farm/pull/21))
- react demo launched successfully! ([#20](https://github.com/farm-fe/farm/pull/20))
- first executable html,css and script demo! ([#19](https://github.com/farm-fe/farm/pull/19))
- implement the basic compilation flow ([#17](https://github.com/farm-fe/farm/pull/17))
- setup node binding and tests ([#8](https://github.com/farm-fe/farm/pull/8))
- set up ci for linting and testing ([#4](https://github.com/farm-fe/farm/pull/4))
- init project with cargo and pnpm

### Fixed

- [#857](https://github.com/farm-fe/farm/pull/857) [#460](https://github.com/farm-fe/farm/pull/460) ([#896](https://github.com/farm-fe/farm/pull/896))
- [#878](https://github.com/farm-fe/farm/pull/878) ([#881](https://github.com/farm-fe/farm/pull/881))
- [#850](https://github.com/farm-fe/farm/pull/850) ([#870](https://github.com/farm-fe/farm/pull/870))
- [#814](https://github.com/farm-fe/farm/pull/814) ([#816](https://github.com/farm-fe/farm/pull/816))
- [#770](https://github.com/farm-fe/farm/pull/770) ([#807](https://github.com/farm-fe/farm/pull/807))
- [#787](https://github.com/farm-fe/farm/pull/787) [#794](https://github.com/farm-fe/farm/pull/794) [#785](https://github.com/farm-fe/farm/pull/785) ([#800](https://github.com/farm-fe/farm/pull/800))
- immutable module not found ([#788](https://github.com/farm-fe/farm/pull/788))
- [#769](https://github.com/farm-fe/farm/pull/769) ([#773](https://github.com/farm-fe/farm/pull/773))
- [#768](https://github.com/farm-fe/farm/pull/768) ([#771](https://github.com/farm-fe/farm/pull/771))
- fix [#760](https://github.com/farm-fe/farm/pull/760) [#761](https://github.com/farm-fe/farm/pull/761) ([#765](https://github.com/farm-fe/farm/pull/765))
- issue 747 ([#758](https://github.com/farm-fe/farm/pull/758))
- bugs ([#710](https://github.com/farm-fe/farm/pull/710))
- config record not work in transform hook ([#707](https://github.com/farm-fe/farm/pull/707))
- [#693](https://github.com/farm-fe/farm/pull/693) ([#695](https://github.com/farm-fe/farm/pull/695))
- bugs when migrate from vite to farm ([#665](https://github.com/farm-fe/farm/pull/665))
- Error in export field lookup algorithm ([#635](https://github.com/farm-fe/farm/pull/635))
- css modules sourcemap gen fail ([#621](https://github.com/farm-fe/farm/pull/621))
- lazy compilation error and windows css error ([#454](https://github.com/farm-fe/farm/pull/454))
- css hmr will always reload the whole page ([#413](https://github.com/farm-fe/farm/pull/413))
- vue migrate bugs ([#357](https://github.com/farm-fe/farm/pull/357))
- module system detection bug ([#345](https://github.com/farm-fe/farm/pull/345))
- add default config for script ([#246](https://github.com/farm-fe/farm/pull/246))
- js plugin cannot use custom type ([#168](https://github.com/farm-fe/farm/pull/168))
- lazy compilation and partial bundling bug ([#44](https://github.com/farm-fe/farm/pull/44))

### Other

- add release-plz to publish cargo crates
- support minify options ([#907](https://github.com/farm-fe/farm/pull/907))
- switch to chokidar ([#886](https://github.com/farm-fe/farm/pull/886))
- Feat/rollup hook compatible ([#842](https://github.com/farm-fe/farm/pull/842))
- optimize cache 4 ([#820](https://github.com/farm-fe/farm/pull/820))
- optimize cache 3 ([#786](https://github.com/farm-fe/farm/pull/786))
- optimize cache 2 ([#785](https://github.com/farm-fe/farm/pull/785))
- _(persistent-cache)_ optimize cache ([#782](https://github.com/farm-fe/farm/pull/782))
- update deps ([#740](https://github.com/farm-fe/farm/pull/740))
- update record viewer ([#731](https://github.com/farm-fe/farm/pull/731))
- resource pot render ([#675](https://github.com/farm-fe/farm/pull/675))
- record add hmr flag ([#696](https://github.com/farm-fe/farm/pull/696))
- Chore/opt vite plugin adapter ([#616](https://github.com/farm-fe/farm/pull/616))
- Feat/js plugin adaptor ([#613](https://github.com/farm-fe/farm/pull/613))
- support buildStart, buildEnd, updateModules js plugins hook ([#574](https://github.com/farm-fe/farm/pull/574))
- version bump ([#563](https://github.com/farm-fe/farm/pull/563))
- js plugin support finish hook ([#513](https://github.com/farm-fe/farm/pull/513))
- update swc and support emotion ([#500](https://github.com/farm-fe/farm/pull/500))
- Support SSR ([#421](https://github.com/farm-fe/farm/pull/421))
- add more features ([#387](https://github.com/farm-fe/farm/pull/387))
- Feat/opt entry output ([#381](https://github.com/farm-fe/farm/pull/381))
- pretty syntax error ([#372](https://github.com/farm-fe/farm/pull/372))
- Optimize tree shake perf ([#369](https://github.com/farm-fe/farm/pull/369))
- solving bugs ([#338](https://github.com/farm-fe/farm/pull/338))
- make the modules order be execution order ([#311](https://github.com/farm-fe/farm/pull/311))
- update css modules hmr and ci yaml ([#299](https://github.com/farm-fe/farm/pull/299))
- Feat/css prefixer ([#294](https://github.com/farm-fe/farm/pull/294))
- do not resolve browser when target env is node ([#238](https://github.com/farm-fe/farm/pull/238))
- bugfix/source-module-graph-error ([#192](https://github.com/farm-fe/farm/pull/192))
- add profiler and optimize resolve speed ([#217](https://github.com/farm-fe/farm/pull/217))
- support ident pat for tree shaking ([#203](https://github.com/farm-fe/farm/pull/203))
- solve issues when add dependencies in HMR ([#194](https://github.com/farm-fe/farm/pull/194))
- Refactor Rust plugin system ([#82](https://github.com/farm-fe/farm/pull/82))
- make query part of id ([#85](https://github.com/farm-fe/farm/pull/85))
- Feat/static assets ([#61](https://github.com/farm-fe/farm/pull/61))
- v0.3.0 support lazy compilation and partial bundling ([#42](https://github.com/farm-fe/farm/pull/42))
- Feat/css hmr ([#36](https://github.com/farm-fe/farm/pull/36))
- Feat/hmr ([#27](https://github.com/farm-fe/farm/pull/27))
- Feat/hmr ([#26](https://github.com/farm-fe/farm/pull/26))
- implement rust hmr interface ([#25](https://github.com/farm-fe/farm/pull/25))
- Refactor build stage to support HMR ([#24](https://github.com/farm-fe/farm/pull/24))
- Feat/dynamic rust plugin ([#22](https://github.com/farm-fe/farm/pull/22))
- adjust ts core arch
