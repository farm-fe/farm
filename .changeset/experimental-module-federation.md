---
"@farmfe/core": patch
---

Preserve CommonJS evaluation order when requiring asynchronous graph modules, and isolate injected CSS style identities and cache entries by runtime namespace. Keep lazy-compilation CSS refresh scoped to the same namespace while invalidating the original module IDs. These fixes support the separate, private experimental Module Federation plugin without changing the public plugin interfaces.
