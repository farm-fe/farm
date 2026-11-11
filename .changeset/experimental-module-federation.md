---
"@farmfe/core": patch
"@farmfe/plugin-react": patch
"@farmfe/plugin-sass": patch
"@farmfe/plugin-tailwindcss": patch
"@farmfe/plugin-vue": patch
"@farmfe/plugin-vue-jsx": patch
"@farmfe/plugin-worker": patch
---

Preserve CommonJS evaluation order when requiring asynchronous graph modules, and isolate injected CSS style identities and cache entries by runtime namespace. Keep lazy-compilation CSS refresh scoped to the same namespace while invalidating the original module IDs. These fixes support the separate, private experimental Module Federation plugin without changing the public plugin interfaces.

Rebuild native compiler consumers against the corrected Rust SDK 2.2.1 crates. Keep the Rust Core ABI and the experimental Module Federation packages unchanged and private.
