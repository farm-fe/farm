# Farm Docs feature consumer

This example uses public `@farmfe/docs` exports, the default theme and its own `site.json`. It does not read Docusaurus configuration.

After building/linking the workspace core, matching native plugins, `packages/docs` and `packages/docs-theme-default`, run `npm run build` here. Serve `build/` at `/guide/`. `expected-routes.json` and `expected-pages.json` are hand-authored assertions for eight output pages.

Four collections form the `guide` group: en/zh × current/previous. IDs associate translations and versions; distinct groups remain independent. A new document exists only in the current version. Missing older-version targets are marked and point to that collection's home. The Chinese current collection explicitly falls back to real English content for `untranslated`; its public metadata identifies the English source instead of presenting it as a real translation.

The current acceptance covers Rust indices, strict missing-translation diagnostics, explicit fallback provenance, static document-link/anchor conversion, static bodies and client bootstrap. The English fallback page links to the Chinese modern document and its explicit anchor. Sidebar, switcher UI, fallback notice/language markup and SEO policy are subsequent implementation steps; this fixture does not claim those features already work.
