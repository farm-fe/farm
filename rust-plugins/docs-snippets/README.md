# @farmfe/plugin-docs-snippets

Rust compilation of explicit documentation snippets and their static highlighting for Farm Docs. The npm entry only returns a native plugin descriptor. Content parsing and transformation do not run through a JavaScript Markdown pipeline.

`DocsSnippetsOptions` accepts an optional `rendererModule` exporting presentation components `CodeBlock` and `CommandTabs`. Optional `highlight.light` and `highlight.dark` each accept `vscode-light`, `vscode-dark`, `inspired-github` or `base16-ocean-dark`. Renderer components control presentation; they do not supply compiler callbacks or runtime syntax highlighting.

For a complete site pass snippet options through the second argument of `defineDocsConfig` from `@farmfe/docs`; the framework coordinates this plugin and its React rendering contract. Do not assume registering snippets alone enables the Docs workflow.

Final tarball/isolated consumer checks remain in progress. Use matching native binaries and retain LICENSE, THIRD_PARTY_NOTICES.md and licenses, including the known embedded syntax resource notices.

The default native palette is `vscode-light` / `vscode-dark`. Coloring follows lexical grammar scopes; it does not run a TypeScript language server or infer arbitrary identifier semantics.
