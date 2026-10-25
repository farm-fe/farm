# @farmfe/plugin-docs-seo

Native Rust SEO integration for Farm Docs. The framework consumes Rust-owned page metadata during its build; this plugin does not parse Docusaurus configuration or delegate compilation to a JavaScript plugin.

The default export accepts `SeoOptions`: `{ planPath }` for a prepared production plan, or `{ development: true }` for the development plan. `planPath` is not accepted with development mode. These options are normally supplied by the `@farmfe/docs` workflow, rather than hand-maintained in site configuration.

Configure site URLs and content metadata through the public Farm Docs configuration. A plugin descriptor alone does not perform static rendering or prove that generated canonical, sitemap or 404 output is correct; inspect the completed site's output and HTTP behavior.

Use matching Core/native binaries. Final package installation and platform verification remain in progress. Preserve LICENSE, THIRD_PARTY_NOTICES.md and licenses for Farm code and known embedded syntax resources.
