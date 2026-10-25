# @farmfe/plugin-tailwindcss

Native Rust Tailwind-style utility generation for Farm. Configuration is JSON-serializable data passed to the native plugin; JavaScript Tailwind configuration files and plugin callbacks are not loaded.

```ts
import tailwindcss from '@farmfe/plugin-tailwindcss';

const plugins = [tailwindcss({ config: { corePlugins: { preflight: false } } })];
```

The public `TailwindCSSOptions` declaration provides `config`, optional `colorVariables`, and a reserved `content` field. Current content scanning follows Farm modules; do not infer support for arbitrary content globs from the reserved field. Stylesheets use Tailwind directives and module content supplies utility classes.

For Farm Docs pass the descriptor through the second argument to `defineDocsConfig`. This implementation's supported configuration is determined by its Rust code and tests, not by the complete JavaScript Tailwind plugin API. Final Farm Docs package/platform and isolated consumer checks remain in progress. Preserve the existing LICENSE and its copyright attributions.
