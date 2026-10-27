# @farmfe/plugin-vue-jsx

## 0.0.1

### Minor Changes

- 8c8f849: Add `@farmfe/plugin-vue-jsx` — a Farm Rust plugin to transform Vue JSX/TSX at build time, ported from `swc-plugin-vue-jsx`. Converts Vue-specific JSX syntax (directives like `v-model`, `v-show`, `v-html`, `v-text`, `v-slots`, custom directives) into Vue 3 runtime calls with zero JS overhead.

  Supports TypeScript props/emits type resolution (`resolveType`), PatchFlags optimization injection (`optimize`), `transformOn` conversion, custom element patterns, object slots, and custom pragmas.

### Patch Changes

- c02ab98: Preserve Vue SFC state for template updates, keep scoped style identities stable,
  and register exported Vue JSX components for development hot updates.
