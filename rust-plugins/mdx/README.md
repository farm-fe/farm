# @farmfe/plugin-mdx

Native Rust Markdown/MDX compilation for Farm. The default npm export loads the platform binary and returns a Rust plugin descriptor; it does not invoke remark/rehype or delegate parsing to Node.

```ts
import mdx from '@farmfe/plugin-mdx';

const plugins = [mdx({ docs: true, format: 'auto' })];
```

`docs` enables the documentation metadata, heading IDs, GFM and admonition path. In Docs mode `format` can be `auto`, `markdown` or `mdx`; auto treats `.md` as Markdown and other matched content as MDX. The shipped `options.d.ts` also declares include/exclude, development, provider and JSX options. Unsupported content should be diagnosed rather than compiled by a JavaScript fallback.

Outside Docs mode, CommonMark is the default. To enable native GFM tables without
the Docs metadata path, use `mdx({ parse: { constructs: { gfmTable: true } } })`.
This forwards an existing native parser option; it does not invoke a JS fallback.

For a complete documentation site use `@farmfe/docs`; a standalone MDX descriptor does not supply routes, static rendering or the Docs CLI workflow. Use matching Core/native binaries. Final package consumer checks remain in progress. Preserve LICENSE and the known embedded resource notices in THIRD_PARTY_NOTICES.md and licenses.
