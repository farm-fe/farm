# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
