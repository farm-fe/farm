---
"@farmfe/core": patch
---

Fix wildcard package export matching and generate safe, distinct filenames for virtual chunks.

Preserve module identity and cached exports when entries import each other or mix inline and separate runtime chunks. Keep public default and named reexports intact under minification, initialize runtime safely without plugins and in workers, and terminate tree shaking for self/cyclic dynamic imports while retaining their entry resource mappings.

Preserve built-in runtime module ids during compiler updates and avoid adding the runtime suffix twice, so glob/watch updates can rebuild runtime modules.

Keep HMR roots internal even when their absolute project paths match external package patterns.
