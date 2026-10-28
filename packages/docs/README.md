# Farm Docs

Farm Docs builds documentation sites with Farm's Rust compiler and native plugins. React runs the browser UI and server rendering; Node orchestrates builds. Markdown/MDX parsing, routes, content validation, resource planning and compilation extensions run in Rust.

This package is under development in the Farm v2 workspace. Published installation instructions and supported binary versions must be verified through the release's isolated package tests; a successful workspace build does not establish installability.

## Configuration

Create `farm.config.ts` in your site root:

```ts
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig({
  title: 'My documentation',
  url: 'https://example.com',
  basePath: '/',
  docs: {
    collections: [{ id: 'main', dir: 'docs', routeBase: '/' }]
  }
}, {
  compilation: { output: { path: 'build' } }
});
```

Add `docs/index.md`, then use `farm start`, `farm build` and `farm preview` through the project's `@farmfe/cli` dependency. These commands use the registered Docs workflow. Adding a native Docs plugin alone does not enable static generation.

Use a dedicated output directory inside the site root. Existing unrelated nonempty directories are not accepted as previous Docs builds. Paths in site configuration resolve from the site root.

## Public entry points

| Import | Purpose |
| --- | --- |
| `@farmfe/docs` | `defineDocsConfig`, generated configuration types |
| `@farmfe/docs/client` | React page/site context, navigation and browser runtime |
| `@farmfe/docs/mdx` | Content component contract and components |
| `@farmfe/docs/snippets` | Code and package manager command components |
| `@farmfe/docs/theme` | Theme slot types and composition |
| `@farmfe/docs/preview` | Static preview server |

Use `locales` and explicit collections for locale/version variants. Give translations the same logical document ID and their collections the same group. Configure `fallbackLocale` explicitly when untranslated pages may use another language. Navigation consumes generated alternate page targets rather than rewriting URL strings.

Override React theme slots through `theme.components`; add stylesheet paths through `theme.customCss`. See the default theme package for the slot contract. The second argument to `defineDocsConfig` accepts native Rust plugin descriptors only. Missing compilation features must be implemented in Rust, without JavaScript/Vite/Webpack compilation fallbacks.

Blog collections support link listings by default. The new opt-in `blogListContent: 'full'` reuses original compiled article components; final browser/native acceptance for that feature is tracked with the current migration, rather than implied by this README.

## Requirements and validation

### Build ownership and performance options

`farm build` keeps the CLI/configuration and React SSR in Node. Preparation and
final verification call Rust directly. A native snapshot owns immutable prepared
content; browser and server compilers lease it through an explicit Core provider.
Each target retains its own module graph, plugin instances, cache identity and
private source receipt. Unknown unobserved inputs still require a conservative
retry; newly controlled reads must retain exact input evidence and pass the final
seal before output can be committed.

Private site configuration can opt into:

```ts
build: {
  highlightRepresentation: 'static-html',
  targetConcurrency: 1
}
```

`highlightRepresentation` defaults to `jsx`. `static-html` preserves the code
block wrapper and interactive MDX, but represents compiler-generated, escaped
token spans as static HTML. Use the default when a custom `pre` renderer needs
to inspect or replace token children. This option does not permit arbitrary raw
HTML from document content.

`targetConcurrency` defaults to `1` to limit memory. `2` explicitly permits both
target compilers to run concurrently; both are drained before failure cleanup.
Choose it only with adequate memory. Persistent caches are isolated by target
and bound to effective configuration, snapshot and native artifact identity.
These options do not imply any performance guarantee; measure cold and warm
builds separately on the actual content and machine.

The package declares Node >=20 and React/React DOM 18 peers. Building native plugins from source additionally requires the toolchain and Node version supported by the current Farm plugin tools. Use Core and native plugins from the same verified distribution; do not substitute arbitrary DLLs or workspace links in an external project.

Rust-generated configuration schema is shipped under `schema/`. Unsupported fields, duplicate routes, invalid references and unavailable content are diagnosed by the native implementation. Preserve the source location and diagnostic code when reporting a failure.

The website's `website/docs/farm-docs/overview.md` and `website/docs/farm-docs/usage.md` introduce the framework and its configuration, theme, build and preview workflow. Final package installation, tarball consumers, platform support and CI results remain separate release checks. This package does not publish or deploy a site automatically.

## Machine-readable output (prerelease candidate)

The candidate `site.ai` configuration opts in to native semantic Markdown, a
`docs-ai-manifest.json` and a small `llms.txt`. No AI output is implied when the
configuration is absent. Eligible document pages get a public Markdown URL and
SHA256 bound to the same build snapshot as their HTML. The default theme shows no
Markdown actions. A custom theme must explicitly render `DocumentActions` to
enable Copy Markdown and View Markdown; an Agent guide link is optional site content.

Unknown MDX components or dynamic expressions fail with `DOCS_AI_UNSUPPORTED`
instead of silently producing incomplete instructions. Generic TSX website pages
do not acquire a fabricated Markdown representation. Copy verifies the exact
Markdown bytes and reports failures rather than claiming success.

This is candidate implementation documentation, not a registry installation or
deployment claim. Matching rebuilt native packages, full fixture/build/preview
acceptance and package publication remain separate checks. Skill installation,
recipes and hosted agent services are not supplied by this option.

See `website/docs/farm-docs/ai-exports.md` for configuration and the publication
contract. The lightweight source tests run with `pnpm --filter @farmfe/docs
test:ai` after workspace dependencies are installed; these tests do not substitute
for native integration acceptance.
