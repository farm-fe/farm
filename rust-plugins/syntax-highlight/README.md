# @farmfe/plugin-syntax-highlight

Native Rust configuration of the shared Farm documentation highlighter. The plugin publishes validated highlighting options for Rust content compilers; it is not a general JavaScript source transformation or browser runtime highlighter.

```ts
import syntaxHighlight from '@farmfe/plugin-syntax-highlight';

const plugins = [syntaxHighlight({
  light: 'inspired-github',
  dark: 'base16-ocean-dark'
})];
```

Both theme fields are optional and accept either theme above. The helper returns a native descriptor. The Docs workflow composes this configuration with its content plugins; registering it alone does not compile Markdown or render a site.

Use matching Core/native binaries. Final distribution and isolated installation verification remains in progress. Preserve Farm LICENSE, THIRD_PARTY_NOTICES.md and licenses: the latter describe embedded syntax/theme resources and their notices, not a claim that every transitive dependency has been audited.
