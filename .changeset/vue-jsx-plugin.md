---
"@farmfe/plugin-vue-jsx": minor
---

Add `@farmfe/plugin-vue-jsx` — a Farm Rust plugin to transform Vue JSX/TSX at build time, ported from `swc-plugin-vue-jsx`. Converts Vue-specific JSX syntax (directives like `v-model`, `v-show`, `v-html`, `v-text`, `v-slots`, custom directives) into Vue 3 runtime calls with zero JS overhead.

Supports TypeScript props/emits type resolution (`resolveType`), PatchFlags optimization injection (`optimize`), `transformOn` conversion, custom element patterns, object slots, and custom pragmas.
