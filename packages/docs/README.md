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

The package declares Node >=20 and React/React DOM 18 peers. Building native plugins from source additionally requires the toolchain and Node version supported by the current Farm plugin tools. Use Core and native plugins from the same verified distribution; do not substitute arbitrary DLLs or workspace links in an external project.

Rust-generated configuration schema is shipped under `schema/`. Unsupported fields, duplicate routes, invalid references and unavailable content are diagnosed by the native implementation. Preserve the source location and diagnostic code when reporting a failure.

The repository's `docs/farm-docs/consumer-guide.md` contains migration and troubleshooting details. Final package installation, tarball consumers, platform support and CI results remain separate release checks. This package does not publish or deploy a site automatically.
