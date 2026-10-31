# @farmfe/plugin-react

Native Rust React/JSX support for Farm. The npm default export returns a platform binary descriptor and serialized `ReactConfig` options.

```ts
import react from '@farmfe/plugin-react';

const plugins = [react({ runtime: 'automatic' })];
```

The shipped declaration defines automatic/classic runtime, importSource, pragma/pragmaFrag, namespace handling, development, refresh and filename options. Configure React runtime dependencies in the consuming project. Farm Docs composes its own React plugin; adding another copy is not required for its default workflow.

Use the native plugin with a compatible, verified Farm Core distribution. Platform declarations do not by themselves establish that a particular architecture's binary was built or accepted. Farm Docs' final tarball and isolated consumer checks remain pending. See LICENSE for Farm code.
