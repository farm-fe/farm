# Use Farm Docs

**Prerelease documentation.** This guide describes the current v2 candidate. Use the commands below only with matching packages from a verified distribution; official package availability is a prerequisite.

## Minimal project

```text
my-docs/
  package.json
  farm.config.ts
  docs/
    index.md
```

Create `docs/index.md` with a heading and some content. Configure the site in `farm.config.ts`:

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

The first argument describes the site. The second argument describes Farm compilation options. `defineDocsConfig` registers the Docs workflow; adding the native Docs plugin alone does not enable static generation.

## Content and configuration

Each collection has an explicit `id`, content `dir` and `routeBase`. Paths resolve from the site root. Use distinct collections for locale/version variants, assign related collections the same `group`, and give translated documents the same logical ID. Configure `fallbackLocale` explicitly when untranslated documents may use another language.

Set a collection's `sidebar` to a site-relative sidebar JSON file to control category and document order. Theme navigation should consume generated targets and alternates rather than derive translated or versioned URLs by string replacement.

The Rust-generated configuration types and schema shipped with `@farmfe/docs` are the authoritative field reference. Unsupported fields and content errors must be corrected rather than hidden by JavaScript compilation fallbacks.

## Theme and extensions

Use `theme.components` to reference a site-local React module and `theme.customCss` for stylesheet paths. The module may export `Layout`, `DocPage`, `BlogPage`, `Navbar`, `Sidebar`, `Toc`, `Footer` and `NotFound`; omitted slots keep their defaults. A default export overrides `Layout`. `NotFound` has its own props type.

```tsx
import type { ThemeProps } from '@farmfe/docs/theme';

export function Footer({ site }: ThemeProps) {
  return <footer>{site.title}</footer>;
}
```

A custom layout must forward the complete props and render slots from `props.components`. Retain `.farm-docs-theme` if using the default theme's shared reading styles. Keep browser-only work in effects so server output and initial browser rendering agree. The default theme supports system, light and dark presentation.

Public imports include `@farmfe/docs/client` for page/site context and navigation, `@farmfe/docs/mdx` for content component contracts, `@farmfe/docs/snippets` for code/command components, and `@farmfe/docs/theme` for slot types. The second argument's `plugins` accepts native Rust plugin descriptors; Vite and JavaScript compilation plugins are rejected by the Docs configuration contract.

## Develop, build and preview

Add scripts to the project using its installed CLI:

```json
{
  "scripts": {
    "dev": "farm start",
    "build": "farm build",
    "preview": "farm preview"
  }
}
```

Run development while authoring. Run the production build before local preview; preview serves the generated static output and is not a deployment. Use a dedicated output directory inside the site root. The workflow refuses unrelated existing nonempty directories instead of treating them as a previous Docs build.

Validate routes, both language variants, resources, navigation, 404 and third-party notices on the actual output. Keep the source location and native diagnostic code when reporting an error. A workspace-source preview, a packaged consumer test and a deployment that installs published packages are separate results.

## Release and installation notes

The package declares Node.js 20 or newer and React/React DOM 18 peers. Use compatible `@farmfe/docs`, `@farmfe/docs-theme-default`, `@farmfe/cli`, Core and native plugin packages from the same verified distribution. The framework's official v2 package release is a separate prerequisite; this guide does not imply that a currently installable release exists.

Once matching official packages are available, install them with normal lifecycle scripts and optional dependencies enabled. Pin the verified versions and commit the package-manager lockfile. Do not use workspace links or arbitrary native binaries to validate an external consumer.

## Migrating a Docusaurus site

Migration uses the site's Farm configuration and explicit component replacements; Farm Docs does not ship a Docusaurus compatibility package or promise full compatibility.

Use `compilation.resolve.alias` for paths such as `@site`. Keep application-specific adapters in the site rather than the framework:

```ts
import { fileURLToPath } from 'node:url';
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig(site, {
  compilation: {
    resolve: { alias: { '@site': fileURLToPath(new URL('.', import.meta.url)) } }
  }
});
```

Replace `@theme/Tabs` and `@theme/TabItem` with named `Tabs` and `TabItem` imports from `@farmfe/docs/mdx`; each item needs an explicit `value` and `label`. Replace `@theme/CodeBlock` with the named `CodeBlock` marker from `@farmfe/docs/snippets`. Its code must be static so the existing Rust snippets compiler can compile and highlight it.

Use `Link`, `useSite`, `usePage` and `useLocale` from `@farmfe/docs/client` for their documented contracts. Rewrite unsupported Docusaurus context, translation and internal theme APIs as site code when needed. An alias only resolves a path: it cannot reproduce another framework's runtime behavior or JavaScript plugin lifecycle. Validate routes, SSR, theme controls, tabs and code output after replacing imports. No Docusaurus runtime or build dependency is required.

## Configuration and customization reference

See the complete [configuration reference](./configuration.md) for fields, defaults and validation rules, and [theme customization](./theme-customization.md) for CSS tokens, React slots, aliases and Rust code rendering.
