# Farm Docs configuration reference

**Prerelease reference.** These are implemented candidate APIs, not a claim that matching official packages are published. Use a verified distribution before following the [usage guide](./usage.md). Rust parses content, validates configuration and plans routes; Node runs the workflow and React SSR.

## Two configuration layers

`defineDocsConfig(site, farm?)` from `@farmfe/docs` takes a `SiteConfig` and an optional `DocsFarmConfig`. Site settings below are **not** Farm compiler settings. Unknown site fields are rejected. Types come from `packages/docs/src/config.generated.ts`; the shipped `schema/input.json` describes JSON shape, while Rust validation also checks URLs, paths, collection relationships and referenced files.

```ts
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig({
  title: 'Example Docs',
  url: 'https://docs.example.com',
  docs: { collections: [{ id: 'main', dir: 'docs', routeBase: '/docs/' }] },
  theme: { navbar: [{ label: 'Guide', to: '/docs/' }] },
  build: { highlightRepresentation: 'static-html', targetConcurrency: 1 }
}, {
  compilation: { output: { path: 'build' } }
});
```

Create `docs/index.md` before running this configuration. The theme and build fields are optional; removing `build` uses the engine's `jsx` representation, not the explicit static-HTML choice above. See [theme customization](./theme-customization.md) for a complete small file layout.

## Site identity, routes and files

| Field | Type / default | Rules and effect |
| --- | --- | --- |
| `schemaVersion` | `1` | Only schema version 1 is accepted. |
| `title` | required string | Nonempty site title. |
| `url` | required string | HTTP(S) **origin**, without credentials, non-root path, query or fragment. Put a deployment prefix in `basePath`. |
| `basePath` | string, `/` | Absolute URL path; normalized with a final slash. |
| `trailingSlash` | `always` (default) or `never` | Public route policy. Files remain static HTML outputs; do not infer output filenames by concatenating URLs. |
| `canonicalOverrides` | array, `[]` | Exact `{ path, canonicalPath }` pairs; both must remain within `basePath`. Changes canonical identity, not a general redirect service. |
| `staticDirs` | array, `[]` | `{ dir, routeBase?: '/' }`. Dedicated existing directories inside the site root; nested/overlapping sources, private tooling paths and symlink/junction static sources are rejected. |
| `linkExceptions` | array, `[]` | Exact `{ collection, id, target, code, reason }` exceptions. Codes are `DOCS_LINK_MISSING` or `DOCS_LINK_ANCHOR`; require a configured collection and a nonempty reason. Not a switch to disable link validation. |

URL paths such as `basePath` and collection `routeBase` must begin with `/` and cannot contain traversal segments, repeated slashes, backslashes, whitespace, `%`, query or fragment. File inputs (`dir`, `sidebar`, `theme.components`, `theme.customCss`) are site-root-relative existing paths of the expected kind; absolute paths and escapes outside that root are rejected. Listing a directory as static publishes its allowed contents: keep credentials, caches and private material outside it.

## Document and blog collections

`docs` defaults to `{ defaultVersion: 'current', collections: [] }`. `blog` is absent by default and, when present, has a `collections` array. Configure at least one collection or `pages.dir`.

| Collection field | Type / default | Rules |
| --- | --- | --- |
| `id` | required string | Unique collection identifier. Identifier/version tags use ASCII letters, digits, `-`, `_`, `.`; empty tags and `..` are invalid. |
| `dir` | required string | Existing source directory inside the site root. |
| `routeBase` | required string | Absolute URL path, e.g. `/docs/`; use a distinct base for each public variant. |
| `exclude` | string array, `[]` | Explicit relative source filenames; not a glob-based ignore language. |
| `format` | `auto` (default), `markdown`, `mdx` | `auto` distinguishes `.md` Markdown from MDX. Explicit format applies to the collection. |
| `locale` | string, `locales.default` | Must be declared in `locales.items`. |
| `version` | string, `docs.defaultVersion` for docs | A docs version tag. Blog collections do not act as versioned docs. |
| `group` | optional string | Connects matching logical documents across locale/version collections. |
| `fallbackLocale` | optional string | Requires an explicit group and a different configured source locale of the same kind/group/version; cycles are rejected. |
| `sidebar` | optional string | Sidebar JSON path for docs. Blog uses its date/tag/archive indexes instead and rejects this field. |
| `admonitionLabels` | object | Optional `note`, `tip`, `info`, `warning`, `danger`, `caution`, `important`, `success` labels. Defaults are `note`, `tip`, `info`, `warning`, `danger`, `caution`, `important`, `success`. An authored title takes priority. |
| `blogListContent` | `links` (default) or `full` | Blog only. `full` renders the original compiled article components in listings. |
| `blogLabels` | object, blog only | Defaults: `title: 'Blog'`, `archive: 'Archive'`, `tags: 'Tags'`, `undated: 'Undated'`, `noPosts: 'No posts.'`; each configured label must be nonempty text. |

Frontmatter can select a document's `id`, `slug`, `title` and `description`. Otherwise IDs/slugs come from relative filenames (`index` maps to its directory); title falls back to the first H1, then the ID. Slugs are safe URL paths, not external URLs. Duplicate logical IDs within a collection and colliding output routes are errors. See the existing blog documentation for post-specific author/date/tag metadata.

### Locales and versions

`locales` defaults to `{ default: 'en', items: ['en'] }`. Items must be unique locale tags; `_` and `.` are not accepted in locale tags. The default must appear in the list. `docs.defaultVersion` defaults to `current` and must be provided by a docs collection when docs collections exist.

```json
{
  "locales": { "default": "en", "items": ["en", "zh"] },
  "docs": {
    "defaultVersion": "v2",
    "collections": [
      { "id": "en-v2", "group": "guide", "locale": "en", "version": "v2", "dir": "docs", "routeBase": "/docs/" },
      { "id": "zh-v2", "group": "guide", "locale": "zh", "version": "v2", "dir": "docs-zh", "routeBase": "/zh/docs/", "fallbackLocale": "en" }
    ]
  }
}
```

Both directories must exist. Give translations the same logical ID. A group cannot contain two collections for the same kind/locale/version. Versioning is explicit collections, not an automatic command that copies content. Use generated alternates and page targets for navigation, not URL string replacement. Fallback content does not become a newly authored translation.

## Sidebars

`sidebar` points to JSON with required `items` and optional reusable `categories`. Item shapes are:

| `type` | Fields |
| --- | --- |
| `doc` | `id`, optional `label` |
| `category` | `label`, `items` |
| `categoryRef` | `id` referring to a key in `categories` |
| `link` | `label`, `href` |
| `autogenerated` | `dir`, relative to the collection |

```json
{
  "items": [
    { "type": "doc", "id": "index", "label": "Welcome" },
    { "type": "category", "label": "Guides", "items": [
      { "type": "autogenerated", "dir": "guides" }
    ] },
    { "type": "link", "label": "Farm", "href": "https://farmfe.org" }
  ]
}
```

The sample requires `docs/index.md` and a real `docs/guides/` document subtree. `categories` maps a reference ID to `{ label, items }`. IDs reference documents, not route URLs. Missing IDs, duplicate appearances and invalid/cyclic category references are diagnosed. Without a sidebar file, generated navigation is used. Top-navigation sections and sidebar filtering are React theme concerns; there is no `navbar.sections` field in `SiteConfig`.

## Standalone pages

`pages` is absent by default. `{ dir: 'src/pages', format: 'auto' }` discovers Markdown/MDX and supported JS/JSX/TS/TSX pages. Script pages use a default React component and a direct static `export const metadata = { ... }`; metadata re-exports or dynamic expressions are not a configuration execution mechanism. The default locale is rooted at `/`, other locales at `/<locale>/` (under the configured base path). Paths and collisions still undergo validation.

```tsx
export const metadata = { title: 'About', slug: 'about' };
export default function About() {
  return <p>Documentation built with Farm.</p>;
}
```

Script `metadata` supports `id`, `title`, `description`, `slug`, `layout` (`default` or `none`) and a static `titleByLocale` map of declared locales to nonempty strings. These are page metadata fields, not arbitrary browser expressions.

## Theme and navigation

| Field | Default | Meaning |
| --- | --- | --- |
| `theme.logo` | absent | `{ src, alt?: '', href?: '/' }`; a public image/link, not a local filesystem URL. |
| `theme.navbar` | `[]` | Flat `{ label, to }` links. Rich menus and icons belong in a custom `Navbar`. |
| `theme.footer` | `{ links: [], copyright: null }` | Links use the same `{ label, to }` shape. Copyright is text. |
| `theme.colorMode` | `system` | `system`, `light`, `dark`; browser selection is persisted by base path when storage is available. |
| `theme.customCss` | `[]` | Site-relative stylesheet files, imported after the framework styles. |
| `theme.components` | absent | Site-relative React theme module; see [theme customization](./theme-customization.md). |

The JSON API does not support Docusaurus `themeConfig`, swizzle paths, a theme package name selector, arbitrary navigation item objects or JavaScript callbacks. Component composition is the extension point. Public navigation/image links accept HTTP(S) URLs, site-absolute paths or nonempty fragments; this is not an arbitrary URL-scheme allowlist.

## Search

No `search` means no configured search. `{ provider: 'none' }` requires absent `options`. Algolia requires all three public, nonempty strings:

```ts
import type { SiteConfig } from '@farmfe/docs';

const search: NonNullable<SiteConfig['search']> = {
  provider: 'algolia',
  options: { appId: 'YOUR_APP_ID', apiKey: 'YOUR_SEARCH_ONLY_KEY', indexName: 'YOUR_INDEX' }
};
```

These values enter browser-visible configuration: never put an admin key here. Configuring Algolia does not provision an account, crawler or a v2 index. Confirm actual indexed URLs, language/version coverage and allowed domains. The default theme has an Algolia UI; the `SearchProvider` React extension allows a site to use DocSearch. DocSearch UI is not selected by an invented `provider: 'docsearch'` value.

Optional `search.openSearch` has required `pageId`, `shortName`, `description`. It requires Algolia and an actual logical standalone search page; `pageId` is not a URL. Short name is at most 16 characters and description at most 1024. Text must be XML-compatible. This generates discovery metadata; it does not create that page's search UI.

## SEO, 404 and machine-readable output

`seo.fallbackIndexing` is `noindex` by default; `index` explicitly allows fallback pages to be indexed. Optional `seo.atom` requires `{ author, updated }`, with a nonempty author and an RFC3339 timestamp with timezone and year 0001–9999. Use a deliberate content update timestamp, not a build-time random value. Canonical URLs and alternates derive from the validated route plan; sitemap/feeds are generated from actual pages, not manually listed routes.

`notFound.labels` is a locale-keyed map of `{ title?, description?, homeLabel? }`. Keys must refer to configured locales. English defaults are `Page not found`, `The requested page could not be found.`, `Return home`. Use a custom `NotFound` slot for layout changes.

`ai` is absent/disabled by default. When the object exists, `enabled` defaults to `true`, `publicationStatus` defaults to `prerelease` (or explicitly `stable`), and `agentGuideUrl` is optional. Use `stable` only for genuinely released content. Eligible document Markdown, manifest, hashes and discovery index share the HTML snapshot; unknown dynamic MDX is an explicit export error. Generic TSX pages do not acquire fabricated Markdown. See [AI exports](./ai-exports.md) for the output/agent contract and current limitations.

## Build execution and compiler options

| Field | Default | Accepted values / scope |
| --- | --- | --- |
| `ssg.concurrency` | `4` | Integer 1–32, Node SSR worker concurrency. |
| `ssg.timeoutMs` | `30000` | Integer 100–300000 milliseconds, SSR execution timeout. |
| `build.highlightRepresentation` | `jsx` | `jsx` or `static-html`. Static HTML is escaped Rust-generated token markup, not arbitrary author HTML. |
| `build.targetConcurrency` | `1` | `1` or `2`, browser/server compiler scheduling; extra concurrency needs memory. |

These are different limits; neither changes the number of documentation routes nor disables verification. This website explicitly uses `static-html`; the framework default remains `jsx` for custom renderers that inspect token children. No browser-side syntax highlighter is required for the Rust-generated code colors.

The second `farm` argument accepts Farm compilation settings and native Rust `plugins` descriptors (`string` or `[string, options]`), plus `snippets`. `defineDocsConfig` owns the workflow and `docs:site` input. Do not supply another workflow, `vitePlugins`, `jsPlugins`, or JavaScript compiler plugin objects. Use `compilation.output.path` for a dedicated site-root output directory (default `build`); an unrelated nonempty directory is not accepted as a previous Docs output.

`snippets` supports `rendererModule` and the legacy `highlight` palette. Prefer the unified `site.code` configuration below so fences, explicit markers and inline signatures use the same theme. The site build setting determines snippets' representation. There is no arbitrary theme-file or JavaScript highlighter callback setting. See [theme customization](./theme-customization.md).

## Validation checklist

- Resolve every referenced directory, sidebar and theme file inside the site root.
- Build the actual content, inspect local preview, translated/version alternates, 404, resources and public AI/SEO output.
- Exercise navigation/search and keyboard interactions after a custom slot change.
- Verify matching published dependencies and native packages separately; workspace success, local tarball installation and deployment are different checks.

## Unified code palette (prerelease candidate)

`site.code` is the single palette input for Markdown fences, explicit `CodeBlock`/`CommandTabs`, and literal inline signatures marked with `data-language`. It does not run a browser highlighter. Optional `light` and `dark` presets accept `farm-light`, `farm-dark`, `vscode-light`, `vscode-dark`, `inspired-github`, and `base16-ocean-dark`. Omitted presets retain the VS Code defaults for compatibility.

```ts
code: {
  light: 'farm-light',
  dark: 'farm-dark',
  tokens: {
    light: { function: '#497774', type: '#526b87' },
    dark: { function: '#9fbebc', type: '#a8b9ce' }
  }
}
```

`tokens.light` and `tokens.dark` optionally override `foreground`, `keyword`, `control`, `function`, `type`, `class`, `variable`, `parameter`, `property`, `string`, `number`, `comment`, `tag`, `attribute`, `constant`, and `punctuation`. Values must be `#RRGGBB`; unknown names and unsafe color expressions fail native validation. `type` covers the type family, including built-in classes and namespaces; an explicit `class` override refines class scopes. More specific categories, such as `parameter` versus `variable`, take precedence where the grammar distinguishes them. The mappings use grammar scopes; they do not add language-server semantic analysis or infer arbitrary symbol identities.

Explicit `site.code` fields override corresponding legacy `snippets.highlight` values. Missing fields fall back to legacy values, then native defaults; token maps merge by display mode and category. The same resolved palette reaches preparation and both build targets. Palette/token changes participate in content-cache identity. `build.highlightRepresentation` remains the single JSX/static-HTML choice. Container background, font, borders and spacing remain `theme.customCss` concerns.
