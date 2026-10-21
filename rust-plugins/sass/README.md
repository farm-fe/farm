# @farmfe/plugin-sass

Rust Sass/SCSS compilation for Farm, using the native plugin's resolver and stylesheet processing. The npm default export returns a native descriptor; it does not delegate compilation to a JavaScript Sass plugin.

```ts
import sass from '@farmfe/plugin-sass';

const plugins = [sass({ additionalData: '$accent: #167347;\n' })];
```

`additionalData` prepends stylesheet content. The shipped `SassOptions` declaration describes this supported public option; do not assume arbitrary options or callback importers from another Sass implementation are accepted.

Farm Docs accepts the descriptor through the second argument to `defineDocsConfig`, alongside site-local Sass imports or custom stylesheets. Use matching Core/native binaries. Final Farm Docs tarball, isolated consumer and platform checks remain in progress. Preserve LICENSE and audit the actual native dependency closure for the target being distributed.
