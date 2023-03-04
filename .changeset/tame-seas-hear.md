---
'@farmfe/runtime': minor
'@farmfe/core': minor
'@farmfe/cli': minor
---

Support lazy compilation and partial bundling

* remove resource pot graph to optimize the compilation speed
* implement partial bundling algorithm
* optimize @farmfe/cli, remove @farmfe/core from its dependencies
* optimize plugin react to skip duplicate module building based on process.env.NODE_ENV