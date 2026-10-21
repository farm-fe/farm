# Native Docs configuration protocol

This page describes experimental Farm Docs in the current workspace. Production builds, the default theme, static preview and the framework development workflow are implemented. Final website migration, current native integration and release packaging remain under validation; workspace behavior does not prove that an independently installed package works.

## Packages and commands

A consumer needs matching `@farmfe/core`, `@farmfe/cli`, `@farmfe/docs` and `@farmfe/docs-theme-default` packages, plus React and React DOM 18. Farm Docs declares Node >=20; native source builds additionally require the Node and Rust toolchains supported by the current plugin tools. Use verified release versions rather than copying `workspace:*` dependencies into an external project. This experimental implementation has not yet completed its isolated tarball installation gate.

Use an ESM package and scripts `"start": "farm start"`, `"build": "farm build"` and `"preview": "farm preview"`. `defineDocsConfig` registers the Docs workflow; installing a plugin tuple alone does not enable the client/server build and static rendering sequence. Preview serves the previous successful build. Locales belong in collection configuration, not Docusaurus `--locale` flags.

Compilation extensions are native Rust plugins. JavaScript/Vite/Webpack compilation fallbacks are rejected; React UI components and thin Node orchestration remain supported. A missing native binary must be fixed through the matching platform package or source build, without substituting a JavaScript compiler.

## Reusable site configuration

Use `farm.config.ts` as the build entry. Farm Docs does not read a Docusaurus configuration:

```ts
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig({
  title: 'My documentation',
  url: 'https://example.com',
  basePath: '/guide/',
  docs: {
    collections: [{ id: 'main', dir: 'docs', routeBase: '/' }],
  },
}, {
  compilation: { output: { path: 'build' } },
});
```

After building and linking the workspace packages, `farm build` invokes the Docs workflow. Rust prepares one immutable content plan, generates the application module and compiles Markdown/MDX. Farm builds client and server outputs from that plan. Node runs React DOM Server and fills the Rust-generated HTML template. The resulting directory contains static page bodies, client hydration code, styles and a build report. Mount this example at `/guide/`; the prefix is part of its public URLs, not an extra physical output directory.

The default theme is provided by `@farmfe/docs-theme-default`. MDX can import React components relative to its source file. A `.md` file uses ordinary Markdown semantics unless its collection explicitly selects `format: 'mdx'`.

### Theme and migration boundaries

`theme.components` points to a site-local module exporting any subset of `Layout`, `DocPage`, `BlogPage`, `Navbar`, `Sidebar`, `Toc`, `Footer` and `NotFound`; omitted slots retain defaults. A default export overrides `Layout`. Import `ThemeProps` and `NotFoundProps` from `@farmfe/docs/theme`. Export a `components` object to override MDX content components. Custom Layouts must pass the complete props to the selected slots, including `listingPosts` when supplied. Stylesheet overrides use `theme.customCss`.

Documentation translations and versions use explicit collections with `group`, `locale`, `version`, `dir` and `routeBase`. Matching logical IDs connect translations. Enable missing-translation fallback explicitly with `fallbackLocale`; use generated alternate targets in custom navigation instead of concatenating locale/version URL segments.

When migrating, preserve a baseline of URLs, anchors, content and resources. Replace framework imports with public Docs APIs or local React components; implement missing compilation capabilities in Rust. Verify static bodies with JavaScript disabled, hydration, navigation, assets, search and development error recovery before switching default commands and removing the old dependencies. A route count alone does not prove content preservation.

Blog collections default to link listings. The new opt-in `blogListContent: 'full'` reuses original article components and their page context; native/browser validation for this addition is still pending in the current website migration.

### Troubleshooting

| Diagnostic or symptom | Action |
| --- | --- |
| Native module cannot load | Check the actual platform package, CPU/libc, Core/plugin compatibility and resolved binary path; use matching verified artifacts. |
| `DOCS_RUST_ONLY` | Remove JS/Vite plugin objects and nested workflows; pass native Rust plugin descriptors. |
| Invalid configuration, duplicate routes or broken anchors | Fix the reported source location; do not suppress validation across the whole site. |
| `DOCS_OUTPUT_OWNERSHIP` or `DOCS_OUTPUT_PATH` | Choose a dedicated output directory inside the site; preserve unrelated existing content. |
| Development compilation fails but old content remains | The last successful generation is retained. Fix the reported error, then verify that the new generation is committed and navigation recovers. |

Report exact commands, diagnostics and actual loaded binary versions, without credentials. Package installation, platform support, CI and full website acceptance remain separate checks.

### Exact collection exclusions

An optional collection `exclude` array omits explicitly named Markdown files from that collection's content index, for example `exclude: ['retired.md', 'drafts/notes.mdx']`. Paths are relative to the collection directory and must identify existing files. Rust rejects patterns, directories, duplicate names, traversal, private tooling paths and symbolic links. Other collections remain independent, and links to excluded content still fail normal link validation. Exclusion paths are private configuration and are not exposed in the public site configuration.

The default is an empty array: missing translations still fail unless an explicit fallback is configured. Excluded files remain part of directory discovery, so adding or deleting them invalidates the prepared listing; removing a declared excluded file requires updating the configuration. Exclusions do not delete source files or create redirects.

Relative Markdown links from translated content can target an explicitly configured fallback page even when its translated source file is absent. Rust resolves the existing fallback's source alias within the current collection and retains its actual public slug, query and validated anchor. It does not infer a public route from a missing filename or switch locale/version implicitly; multiple matching fallback documents are an error.

### Localized standalone page titles

Standalone page metadata can provide a static `titleByLocale` object, for example `export const metadata = { title: 'Search the documentation', titleByLocale: { zh: '在文档中搜索' } }`. Rust selects the current locale's override, otherwise retaining the ordinary title or heading fallback. The final browser and static HTML title consume the same Rust SEO plan. Markdown standalone pages accept the same map in frontmatter.

Keys must name declared site locales, and values must be nonempty strings without control characters. Script metadata rejects dynamic values, spreads, computed keys and duplicate locale keys. This field does not choose content routes, change the site language, or apply to documentation/blog collections.

The second argument accepts Farm options and native Rust plugin descriptors. JavaScript and Vite compilation plugins are rejected. Existing nonempty output directories must contain a previous Farm Docs build report; Farm Docs refuses to replace unrelated content. Output and staging paths must stay inside the site and cannot pass through symlinks or Windows junctions.

Content IDs must be unique within a collection. An explicit frontmatter `id` keeps the opaque page key stable when its source file is renamed. Case-equivalent and Unicode-normalization-equivalent URLs are rejected without silently rewriting their authored spelling. Output paths must also be portable to Windows: device names, trailing dots, empty middle segments and invalid filename characters are rejected before publishing.

`examples/docs-minimal` verifies two static pages, reading with JavaScript disabled, styles under a base path and hydration of an imported counter. `examples/docs-features` additionally exercises a four-collection language/version index and explicit fallback. The default theme renders Rust sidebars, previous/next links, language/version alternatives and blog metadata.

## Experimental development server

`farm start` now runs the native Docs, SEO and React plugins through Farm's incremental server. Markdown edits update the current document through HMR. Adding a document also updates the Rust-generated sitemap. Site configuration and its imported dependencies reload through an independent candidate compiler. Configuration parsing or native content-plan failure retains the previous usable generation; repairing the configuration resumes updates. Successful configuration reload replaces the page document.

Changing to a different HTTP port prepares a new service before retiring the old one. A port conflict or candidate compilation failure keeps the old service available. After a successful handoff, the browser moves to the new origin while retaining its path, query and fragment; subsequent edits use the new HMR connection. The programmatic handle follows the published service. During a listener handoff or after close, its URL retains the last successful published address. Response headers can change on the same port: the new headers become visible with a successful candidate compilation, failed candidates retain the old headers, and removed headers disappear after repair. HTTP proxy routes can change on the same port after successful candidate compilation, including WebSocket routes; failed candidates retain the previous proxy. Retired proxies release their requests and sockets. When HMR shares the HTTP listener, same-port HTTP/HTTPS changes prepare the native candidate first, pause the old listener, and restore the original compiled service if the new listener fails. Invalid certificates retain the previous service. Fixed independent HMR ports also support this handoff and configuration changes: candidate binding is deferred, failed candidates restore the previous listeners, and the browser reconnects to the committed HMR endpoint. HTTPS proxy changes can replace the HTTP/2 listener with HTTPS HTTP/1 and restore HTTP/2 when removed. Certificate rotation and root changes also prepare a candidate before retiring the previous service; failed candidates retain the successful service. The full development acceptance matrix, including cancellation and recovery combinations, remains in progress.

For programmatic lifecycle control, import `startDocsDevelopment` from `@farmfe/docs`:

```ts
const controller = new AbortController();
const server = await startDocsDevelopment(site, {
  root: process.cwd(),
  server: { host: '127.0.0.1', port: 9000 },
}, { signal: controller.signal });
console.log(server.url); // Server origin; content URLs follow the Rust site plan.
await server.close();
```

`close()` is idempotent. Aborting the signal also closes the server. An already aborted signal rejects startup. This API uses native compilation plugins and accepts the same native-only Farm options as `defineDocsConfig`.

Docs shares one process-level SIGINT/SIGTERM handler across development handles.
Termination waits for every registered Docs owner, including pending configuration
handoffs and native cache drains. Repeated signals reuse that shutdown; a cleanup
failure is reported and exits with a nonzero code. Explicit API close and failed
startup unregister their owner. Core embedders can pass `handleSignals: false` to
`Server.createServer` when an external owner provides process termination handling;
the default Core behavior remains enabled.

Runtime watchers treat project paths literally by default, including brackets in
directory names. Generated output and native cache directories are excluded using
directory boundaries, while user `watch.ignored` rules still apply. An explicit
`watch.disableGlobbing` setting takes precedence.
Runtime watchers also wait for writes to settle before delivering changes,
including rapid error repairs on Linux; `watch.awaitWriteFinish` can override this.

The generic Core runtime also exposes `ServerRuntime.replaceCompiler(candidate)`. It serializes candidate compilation with updates, publishes defensive copies of the candidate resources and changes compiler identity only after successful compilation. Failure retains the active compiler and resources. Closing during replacement waits for native work and prevents candidate publication. `Server.createServer` accepts a `reloadConfig` callback in runtime mode; configuration-file, configuration-dependency and environment-file events use this lifecycle callback before creating a new compiler. It is separate from compilation plugin hooks.

Development uses `site.basePath` as Farm's resource public path, including the documentation manifest. Awaiting the development handle's `close()` also waits for owned Rust background cache writes. Failed candidates and retired compilers are drained before their ownership is released. Cleanup attempts all network resources and reports cache or network cleanup failures together; repeated close calls share the same result. Direct Core compiler consumers can call `await compiler.flushCache()` after they have stopped admitting compilation and updates. This waits for already scheduled cache tasks and reports task panics; existing disk-store IO diagnostics retain their current logging behavior.

On startup and WebSocket connection, the development client checks the Rust snapshot identity of the server's published resources. This recovers updates missed while a full page reload establishes its new connection, including an apparently valid but stale application and manifest. A different published generation permits one automatic reload per target snapshot and path; repeated stale delivery shows a retry button instead of a reload loop. The retry button permits an explicit fresh attempt. Normal subsequent HMR still preserves the current document, and manifest validation remains strict.

`RuntimeServerOwner<T>` owns complete service instances on a separate lifecycle queue. `replace(prepare)` passes an AbortSignal to preparation; preparation must clean up partial allocations until it returns a usable instance. Preparation failure retains the active service. Success publishes the candidate and closes the previous instance. A retirement cleanup failure is reported after publication and the new service remains active. `close()` cancels preparation, begins closing owned instances immediately, waits for late candidates and attempts all remaining cleanup. `Server.createServer` also accepts `onConfigChange` for delegating configuration events to such an external owner, outside the native update queue.

Cancellation remains connected to a prepared replacement while the previous WebSocket finishes its reload message. Closing during that interval closes both transports and waits for the pending lifecycle operation; a late completion cannot publish another service.

## Public URL and canonical policy

`trailingSlash` accepts `always` (the default) or `never`. The mounted site root keeps its slash; other document, standalone page, blog article and generated-view URLs follow this single Rust policy. Physical output still uses directory `index.html` files, so changing public URL style does not change deployment file locations.

Each public page exposes separate `routePath` and `canonicalPath` values. By default they are identical. Use explicit `canonicalOverrides` for a historical difference such as a locale homepage:

```json
{
  "basePath": "/guide/",
  "trailingSlash": "never",
  "canonicalOverrides": [
    { "path": "/guide/zh", "canonicalPath": "/guide/zh/" }
  ]
}
```

Paths include the public base prefix. The source and target must be actual pages with their exact public spelling; a trailing-slash variant is allowed. Escapes, external URLs, paths outside the mount, duplicate source pages and stale rules fail with `DOCS_CANONICAL_*` diagnostics. An explicit cross-locale canonical may point directly to an existing target's final canonical URL; chains and cycles are rejected. Overrides do not create routes or move files, and their input rules remain private.

Rust finalizes routes before generating alternates, sidebar navigation and compiled content links. Public links accept either trailing-slash spelling and emit the selected route while preserving query and fragment. React SSG consumes `canonicalPath` directly. Complete 404 generation, sitemap/hreflang, hosted redirects and client navigation have separate acceptance gates.

## Collection groups, translations and versions

Declare `group` on collections that represent the same logical documentation set. Collections without a group stay independent, even when their IDs or source paths look similar. A group has one collection per declared locale/version combination and requires a complete matrix of the locales and versions represented within that group.

```json
{
  "id": "zh-current",
  "group": "guide",
  "dir": "content/zh-current",
  "locale": "zh",
  "version": "current",
  "routeBase": "/zh/docs/",
  "fallbackLocale": "en"
}
```

Every document ID must have a translation in the group's other locales for the same version. The default is a `DOCS_TRANSLATION_MISSING` error. An optional `fallbackLocale` selects a configured locale in the same group/version; it cannot name itself, an unknown source or form a cycle. The requested document must actually exist in that source collection: fallback does not silently turn an already synthesized page into a new source.

Rust emits the target page from the true source content, with public `fallbackSource` metadata identifying the original collection, key, locale, version and URL. Source filesystem paths remain private. `fallbackLocale` is compiler policy and is omitted from public site configuration.

Public page `alternates` contains the precomputed locale/version targets. Versions may add or remove documents: a missing document in another version has `missing: true`, no invented document key and the target collection's home URL.

The default theme lists the current version's languages in `locales.items` order and the current language's versions in Rust collection order. Empty and single-choice controls are hidden. All targets are ordinary links and work with JavaScript disabled; client navigation retains the same document. Missing-page links explicitly say that the page is unavailable and lead to the actual version home. A real fallback page instead identifies its content's source language, and its body includes a link to the original page. Current choices expose `aria-current`. Blog articles and generated views use their own Rust alternatives, even when documentation collections share a group name. Complete mobile presentation and SEO have separate acceptance gates.

The collection home must exist: Rust prefers a real document at the collection root, otherwise the first actual page in public-URL order. An empty grouped collection fails with `DOCS_COLLECTION_EMPTY`; an alternate never silently points at an unpublished root URL.

## Static documentation links

Farm Docs retains the Rust syntax tree until every document and heading is indexed. Relative `.md`/`.mdx` links are resolved from the original source file and emitted as public URLs with `basePath`, query and fragment preserved. Missing documents or anchors fail with `DOCS_LINK_MISSING` or `DOCS_LINK_ANCHOR`; diagnostics include the original filename, one-based line/UTF-16 column, target and source snippet.

Static HTML `id` attributes also participate in anchor validation. Raw Markdown HTML is parsed in Rust with original source positions. A file reused by collections with different formats or locales keeps each collection's semantics; links do not silently adopt the last collection's language.

Relative file links cannot implicitly change locale/version (`DOCS_LINK_CONTEXT`). Use an explicit public URL for an intentional cross-context link. On an explicit fallback page, Rust uses the real source file to locate the referenced document, then selects the target locale's precomputed alternate and checks that target's actual anchors. `examples/docs-features` includes a fallback-page link to a Chinese document with a stable explicit anchor.

Encoded traversal fails with `DOCS_LINK_ESCAPE`. Ordinary external HTTP(S), protocol-relative, mailto and telephone links are preserved. Dynamic JSX expressions and custom components' runtime link props are not presented as statically verified Markdown links.

Root-relative `.md`/`.mdx` source aliases use each collection's route base and the real source filename, with `basePath` applied once. For example, `/docs/start.md#install` can resolve to `/guide/docs/getting-started/#install` when frontmatter overrides the slug. The alias is not inferred from the document ID. Explicit fallback pages expose their source filename under the target collection's namespace and still validate the target's actual anchors.

An actual public route takes precedence over a source alias. Multiple owners of the same alias produce `DOCS_LINK_AMBIGUOUS` with the original reference position and candidate public URLs; Farm does not choose whichever collection was scanned last. These aliases rewrite compiled content links only: they do not create `.md` endpoints or redirects.

Asset references currently receive `DOCS_RESOURCE_PENDING` diagnostics; asset graph conversion and validation remain a separate implementation step. This experimental link slice does not establish complete asset or navigation support.

### Exact historical exceptions

Optional site `linkExceptions` entries contain `collection`, canonical document `id`, raw `target`, exact `code`, and a required `reason`. Only `DOCS_LINK_MISSING` and `DOCS_LINK_ANCHOR` can be configured. Rust first runs the normal resolver, then matches all fields; no wildcard, regex or global ignore is supported. Rules cannot suppress protocol, encoding, path escape, ambiguity or locale/version errors, and do not carry across collections sharing a source file.

Anchor exceptions still produce the real target's canonical public URL with the original query/fragment. Missing-document exceptions preserve the unresolved authored target; they do not generate pages or redirects. An exception acknowledges a recorded issue rather than repairing the link. Fixed or removed links cause `DOCS_LINK_EXCEPTION_UNUSED` until the stale rule is removed. Invalid source IDs produce `DOCS_LINK_EXCEPTION_SOURCE`.

The private plan retains the full original diagnostic and reason. Rust also generates `publicDiagnostics` for the published build report, containing public document identity and source line/column without local filesystem paths or configuration reasons. The public site configuration and generated application omit `linkExceptions`.

The native plugin separates these input modes:

- `site`: validate site configuration in Rust and expose its public projection.
- `site` with `prepare: true`: Node-only preflight producing a private compiled content plan.
- `site` with `development: true`: low-level native development integration. Rust prepares a working content plan and refreshes its registry and loaded content modules during incremental updates. This mode does not start a server or implement the public `farm dev` workflow by itself.
- `planPath`: verify a prepared plan and source hashes, then expose compiled content and the generated application to child builds.
- `snapshotPath`: load a previously prepared schema-versioned snapshot, as used by the integration prototype.

Supply exactly one of `site`, `planPath` or `snapshotPath`. Missing or ambiguous modes return `DOCS_OPTIONS`. `prepare` requires `site` and a Node target; a browser preflight fails with `DOCS_PLAN_TARGET`.

`development` requires `site` and cannot be combined with `prepare`. Its working plan is not a public commit receipt: a later component compilation can still fail. Pending content and registry invalidations are retried until the native update completion hook is reached. A server integration must publish resource buffers only after the complete synchronous update succeeds.

The native `@farmfe/plugin-docs-seo` plugin accepts either `{ planPath }` for a prepared production plan or `{ development: true }` alongside the Docs development plugin in the same compiler. The latter reads the current validated Rust plan through private compiler context on every finalization; it does not reread an old plan file or compute routes in JavaScript. Missing, malformed or outdated records fail without replacing its previous SEO resources. Neither the private plan nor its filesystem paths are emitted as a browser resource.

## Site configuration preflight

After linking the workspace plugin, a Farm configuration can compile a configuration entry:

```ts
import { defineConfig } from '@farmfe/core';
import docs from '@farmfe/plugin-docs';

export default defineConfig({
  compilation: {
    input: { config: 'docs:site' },
    output: { targetEnv: 'node', format: 'cjs' },
  },
  plugins: [docs({
    site: {
      schemaVersion: 1,
      title: 'My documentation',
      url: 'https://example.com',
      basePath: '/',
      docs: {
        collections: [{ id: 'main', dir: 'docs', routeBase: '/docs' }],
      },
    },
  })],
});
```

The `docs` directory must exist. This mode produces a JavaScript configuration module and `docs-public-config.json`; it does not yet render documentation pages.

Rust applies the default locale `en`, document version `current`, and content format `auto`. `auto` uses Markdown semantics for `.md` and MDX for `.mdx`; use `format: 'mdx'` on a collection to preserve legacy `.md` content containing MDX expressions or JSX.

Site `url` is an absolute HTTP(S) origin without credentials, query, or fragment. `basePath` supplies the path prefix. Collection IDs must be unique and configured locales must be declared. Collection directories, sidebar files, custom CSS, and theme component inputs are relative to the site root; missing inputs, wrong file types, traversal, and resolved paths outside that root fail validation.

## Sidebar JSON and navigation data

A document collection can set `sidebar: 'sidebar.json'`, relative to the site root. The file uses Farm's JSON protocol, not executable Docusaurus sidebar configuration:

```json
{
  "items": [
    { "type": "doc", "id": "index" },
    { "type": "category", "label": "Learn", "items": [
      { "type": "doc", "id": "getting-started", "label": "Begin" }
    ] }
  ]
}
```

Supported entries are `doc`, inline `category`, `categoryRef` into a top-level `categories` map, `link`, and `autogenerated` with a collection-relative `dir`. Omitted sidebar configuration generates the complete physical directory tree: index documents first, then stable filename/directory order. Document IDs honor frontmatter overrides. To preserve custom ordering, use explicit entries; automatic trees currently do not read `_category_.json` or frontmatter ordering fields.

Rust rejects missing documents/categories, duplicate documents, category cycles, invalid URLs and directory escapes. It generates public sidebar data plus each listed page's `previous`/`next` in sidebar order; external links do not enter that sequence. Omitted documents remain routable without previous/next. Each locale/version collection has a separate sequence, including its real fallback pages.

Sidebar bytes participate in the private prepared snapshot. A changed sidebar invalidates a child build with `DOCS_SNAPSHOT_CHANGED`. The generated application exports public `sidebars`; source file paths and hashes stay private. This is a data contract: default-theme sidebar presentation and development watching have separate implementation gates.

## Public and private configuration

`docs:site` exports the Rust-generated public configuration. It contains site metadata, locale/version defaults, public collection identities and route bases, navigation, and declared public search settings.

Collection directories, sidebar file inputs, `theme.customCss`, and `theme.components` stay in private Rust compiler state. They are omitted from the configuration module and JSON resource. The `site` object accepts serialized data; it is not a JavaScript compiler callback.

Diagnostics include a code and field path, for example `DOCS_CONFIG_VERSION` for an unsupported protocol version or `DOCS_CONFIG_PATH` for an invalid input. Unknown fields produce `DOCS_CONFIG_PARSE`.

Content frontmatter diagnostics retain the original file, one-based line and UTF-16 column of the YAML value. Nested fields and sequence entries use their actual positions; alias expansion points to the alias use. A missing required property points to its nearest existing parent container. Rust keeps this source index private and does not execute page modules to infer metadata.

JSON Schema and TypeScript declarations are generated from the same Rust types. Input defaults are applied during validation; the public payload requires resolved language, footer, and logo fields instead of silently applying those defaults again. The generated TypeScript models structural fields, optional values, enums, and the protocol version. Filesystem, URL, and collection business rules are validated by Rust.

The maintainer schema example in `rust-ecosystem/docs-plan` writes input, public, private-plan and sidebar JSON Schemas plus the TypeScript declarations used by `@farmfe/docs`. They share one strict Rust schema renderer. The private plan contains source paths, compiled modules and maps; it is not emitted into client/server child outputs or the published site.

## Local content resources

Relative Markdown images and download links are resolved in Rust from the real content file, including fallback content's original source. A resource may live outside its collection when it stays inside the site root. Query strings and fragments are retained. Percent-encoded Unicode filenames are supported; encoded traversal/separators, missing files, directory targets and symlink/junction ancestors fail with original-reference diagnostics.

The private prepared plan records source files and digests. Rust generates opaque public resource names under `assets/docs/`, adds resource modules to the Farm application graph, and emits their bytes through the native resource hook. Both child builds verify the snapshot; changed resource bytes invalidate it. A new build produces a new URL after content changes. Node only consumes the resulting manifest/resources.

Actual indexed pages take priority over download classification even when a public page slug contains a file extension. Full dependency snapshots and development watching retain separate implementation gates. Missing local resource references fail instead of producing a pending diagnostic.

Declare static directories with Farm's own serialized configuration:

```ts
staticDirs: [
  { dir: 'static' },
  { dir: 'downloads', routeBase: '/downloads/' }
]
```

Directories must be dedicated, non-overlapping directories inside the site root. The site root itself, private tooling directories, and symlink/junction sources or descendants are rejected. Rust emits every file, including unreferenced files, preserving its path under the declared `routeBase` (default `/`). Public URLs add `basePath` once and percent-encode filename segments; Unicode, spaces, query strings and fragments are supported. The source directories stay private.

Root-relative Markdown images/downloads, `theme.logo.src`, and blog author images resolve against these files. With `basePath: '/guide/'`, `/img/logo.svg` becomes `/guide/img/logo.svg`; an already mounted reference is not prefixed again. External image URLs are retained. Missing local files fail with source diagnostics. Relative references to the same static source use its named output too.

Rust checks static file bytes and directory listings against the prepared snapshot. Changed bytes and added/deleted entries invalidate it. Page files, reserved metadata, and actual compiler outputs share a collision check that rejects case-equivalent names and file/descendant conflicts before emission. Native tests verify exact bytes, browser image decoding, Unicode URLs, junction rejection/recovery and retention of the previous successful output on collisions. Default-theme logo presentation and development watching have their own remaining gates.

Imported React components, CSS Modules, images and fonts use Farm's native graph. Supply a native Sass plugin to compile SCSS Modules. Both Docs targets use browser asset URLs, so SSR image imports do not become local `file:` URLs. Root-relative CSS `url()` references use the declared static namespace and receive `basePath` once, preserving query strings and fragments. Ordinary relative assets retain native processing. Stylesheet imports remain stylesheet dependencies rather than opaque image resources.

With source maps enabled, CSS resource diagnostics trace existing maps back to the authored SCSS. Minified CSS maps retain column mappings so a single generated line can identify multiple original files. Production tests cover nested interactive components, actual font loading, image bytes, CSS import order, component/style/image rebuilds and missing-resource recovery. These checks do not establish development watching or a complete build-time dependency snapshot.

## Standalone pages

The browser runtime uses Rust's page and URL-alias registry for same-document navigation, including trailing-slash variants and Unicode URLs. Locale/version context, title, description, canonical and document language update on navigation. Query strings and fragments are retained; history, focus and scrolling are handled within the mounted application. Modified clicks, downloads, external links and unregistered addresses retain browser behavior. Explicit reloads still load the generated HTML.

Rust generates explicit asynchronous content loaders for the browser and retains synchronous content for SSR. The browser prepares the current page before hydration and loads destination modules during navigation. The native manifest associates each page with actual emitted scripts and styles; generated HTML includes page styles for reading without JavaScript. A browser loader retains global styles and activates page styles in native order, so returning to a page restores its cascade. The `Link` component and ordinary registered anchors share browser navigation behavior.

First-load module errors retain the SSR content and expose a retry action. Full chunk/network failure acceptance, asynchronous navigation race tests, default-theme navigation and complete SEO output retain separate implementation gates.

Navigation commits its URL and Head after the page renders successfully. A component render or layout failure shows a retry action and recovers the last committed page; a failed history navigation returns to its previous cursor. Recovery may recreate component state. A later component error on an already committed page recovers that same page rather than an older route. If recovery also fails, the retry message remains available. Network chunk failure handling belongs to the asynchronous loading gate.

Set `pages: { dir: 'src/pages', format: 'auto' }` to index Markdown/MDX and React JS/JSX/TS/TSX pages. The same source directory serves each declared locale; components can use `useLocale()` from `@farmfe/docs/client`. The default locale uses the site base path, and other locales add their locale segment. Pages have no document version.

React pages export a default component and may directly export a static `const metadata` object with `id`, `title`, `description`, `slug` and `layout`. Import `PageMetadata` from `@farmfe/docs` for TypeScript validation. Rust extracts literal fields without executing the module; dynamic expressions, spreads, shorthand, unknown fields and metadata re-exports fail with source diagnostics. Farm's Rust pipeline compiles the original typed source.

Files or directories starting with `_` are excluded from page scanning and can hold imported components. Scripts with only data exports do not create routes. Invalid syntax still fails. `layout: 'none'` skips the theme layout while retaining the page context and site styles. Markdown links such as `./index.tsx#home` resolve through the Rust index and can validate literal intrinsic JSX IDs. Dynamic component IDs and runtime links require browser verification.

Dependency snapshots, development watching, full navigation and website migration have separate acceptance gates; this page API does not imply those gates are complete.

## Theme components

`theme.components` selects an explicit local theme module. Export any of `Layout`, `DocPage`, `BlogPage`, `Navbar`, `Sidebar`, `Toc`, `Footer` and `NotFound`; omitted exports keep the default components. A default export remains an alternative to `Layout`. An explicit invalid component, including `null`, fails with a `DOCS_THEME` diagnostic naming the slot.

```tsx
import type { ThemeProps } from '@farmfe/docs/theme';

export function Navbar({ site }: ThemeProps) {
  return <header>{site.title}</header>;
}

export const components = {
  h2: (props) => <h2 className="project-heading" {...props} />
};
```

The separate `components` export customizes Markdown/MDX content components. It does not replace the named layout slots. `ThemeProps` includes `site`, `page`, `children`, resolved `components` and `data` containing Rust's public `pages`, recursive `sidebars` and `blogIndexes`. A custom Layout can invoke these resolved slots with the same props. No hidden theme aliases or source-directory scanning is involved.

The default sidebar consumes the Rust tree and marks the active page by key. Previous/next links and translation fallback notices use Rust's public metadata. `BlogPage` also applies to generated blog indexes, archives and tag views. A `layout: 'none'` page retains its provider and content mapping while skipping the Layout.

Rust also supplies `data.home` for brand and return-home links. It selects an actual site root first, then prefers default-locale/default-version collections, their actual root, first recursive sidebar document, or first ordered route. A logo's default `href: '/'` selects this actual home. Explicit local overrides and same-origin absolute URLs must resolve to existing pages and anchors; query strings and fragments are preserved. Origin comparison normalizes host case and default ports. External HTTP(S) targets remain external.

`NotFound` receives separate `NotFoundProps` containing `site`, `data` and `components`; it does not receive a fabricated content `page` or article children. Its missing-page provider supports `useSite`, `useLocale` and `useColorMode`. `usePage` and `useVersion` fail explicitly with `DOCS_PAGE_CONTEXT`, since no content page exists. `data.notFound` supplies the Rust-selected `locale`, `routeBase`, `home`, `heading`, document `title`, `description`, `homeLabel` and `robots`. The default component renders these labels and links to a real localized home.

Unknown URLs within the site mount render the localized missing view within the current document. The runtime uses Rust's manifest membership to retain native navigation for registered public resources, including extensionless files. Explicit downloads, external links, new-window targets and modified clicks keep browser behavior. Foreign history entries and URLs outside the mount load the actual document. Direct requests and refreshes receive HTTP 404 from Docs preview; a production host must configure its own physical 404 fallback.

The default layout preserves its sidebar and table of contents with JavaScript disabled, including narrow screens. With JavaScript, viewports at or below 48rem move these controls into a native modal navigation drawer. Open navigation, Close navigation, Escape and backdrop dismissal are supported; keyboard focus stays within the open drawer and returns to its trigger on dismissal. Following a page or fragment link closes the drawer before navigation restores content focus. Back/Forward also dismisses an open drawer. Closing, resizing or unmounting restores the previous body scroll policy. Custom Sidebar and Toc exports remain the drawer's actual components.

The default toolbar offers a Color theme selector for system, light and dark after mounting, using the public color-mode state below. The desktop layout includes a keyboard-visible skip link to the main content. These interactions do not change Rust's route registry or infer missing pages.

## Output replacement failures

Farm Docs builds candidates in a private staging directory and replaces the previous output only after compilation and rendering succeed. On Windows, directory rename retries `EPERM`, `EACCES` and `EBUSY` up to seven attempts with a total scheduled wait of 1.575 seconds. Other errors fail immediately. Retry diagnostics include the operation and attempt; persistent failures remain errors.

If installation fails, Farm Docs attempts to restore the previous output. If restoration also fails, an `AggregateError` retains both errors and identifies the retained backup directory; staging is preserved for recovery. A backup is recorded only after the old output has actually moved. Cleanup failure also retains the original build error and states whether new output was committed. These rules do not promise an uninterrupted atomic directory switch for concurrent readers. Full concurrent-build, crash recovery and production transaction acceptance retain their implementation gates.

## Content tabs

Import `Tabs` and `TabItem` from `@farmfe/docs/mdx`. Each direct `TabItem` has a nonempty unique `value`, a `label` and its React content; optional `disabled` items are skipped by keyboard selection. `defaultValue` must identify an enabled item, or the first enabled item is selected. Invalid children, duplicate values and invalid defaults fail with `DOCS_TABS`.

```mdx
import { Tabs, TabItem } from '@farmfe/docs/mdx';

<Tabs label="Package manager" defaultValue="npm" groupId="package-manager">
  <TabItem value="npm" label="npm">Run npm install.</TabItem>
  <TabItem value="pnpm" label="pnpm">Run pnpm install.</TabItem>
</Tabs>
```

With JavaScript disabled, all panels remain readable and labels link to their panel fragments. After mounting, the controls become accessible tabs with one active panel, arrow-key navigation, Home/End and stable tab/panel IDs. Nested tabs remain independent.

An optional `groupId` synchronizes matching choices within a document and through localStorage across tabs and reloads, isolated by site basePath. A choice absent from another group is ignored there. Removing the stored key or clearing localStorage resets other open tabs to their configured defaults. sessionStorage events are ignored. Denied storage leaves local interaction usable. Persistence is applied after mounting, preserving the SSR and first hydration tree.

## Color mode and browser-only content

`useColorMode` from `@farmfe/docs/client` returns `colorMode` (`system`, `light` or `dark`), `resolvedColorMode` (`light` or `dark`) and `setColorMode(mode)`. The initial selection comes from `theme.colorMode`; the default is `system`. Consumers share their provider's state, and independent providers for the same site synchronize changes.

```tsx
import { useColorMode } from '@farmfe/docs/client';

export function ThemeSwitch() {
  const { colorMode, resolvedColorMode, setColorMode } = useColorMode();
  return <button onClick={() => setColorMode(resolvedColorMode === 'dark' ? 'light' : 'dark')}>
    Theme: {colorMode}
  </button>;
}
```

The browser stores the selection under `farm-docs:color-mode:<basePath>`. Invalid stored values fall back to the site default; unavailable storage does not prevent changing themes. System mode follows media preference changes; explicit modes override them. Other tabs synchronize through storage events, and mounted providers remove their media/storage listeners when unmounted.

The Rust Docs plugin emits a standalone browser bootstrap resource and inserts its blocking script before styles and application scripts through the HTML AST. It applies a valid saved preference before the SSR body appears. Fixed light/dark defaults also appear in static HTML; system mode follows CSS media preferences without JavaScript. React's first render uses the deterministic SSR state, then synchronizes browser preferences without suppressing hydration warnings.

`ClientOnly` renders its stable `fallback` on the server and during the first browser render. Its child function runs only after mounting, so it can use browser DOM APIs. This does not defer the rest of the page's SSR content. The default theme's visible selector uses this same mounted-state discipline.

## Blog article metadata and indexes

Set `blogListContent: 'full'` on a blog collection to include its original article components in the listing. The default, `'links'`, keeps a compact link list. Archive and tag views remain link indexes. This option is rejected on documentation collections.

Rust selects the articles and generates their module dependencies. Server rendering and lazy client navigation reuse each original compiled Markdown/MDX component, including its resolved links and assets. Content components receive their article's page context rather than the listing page. Custom layouts must forward `listingPosts` with the other `ThemeProps`; a custom `BlogPage` can render each `{ page, children }` entry. Full listings load all listed articles before navigation commits, so use link mode when loading every article would be too expensive. No source text is copied or compiled in Node.

`blog.collections` uses the same explicit directory, route base, format and locale/group settings as document collections. Blog collections do not accept document versions or sidebars. Rust compiles actual Markdown/MDX article bodies and supplies each page's `blog` metadata with `date`, optional `publishedAt`, `authors` and `tags`.

Top-level `blog/index.md` remains the `index` article, distinct from the collection listing. Dated filenames such as `2024-02-29-release.md` default to `2024/02/29/release`; explicit frontmatter `slug` overrides the path. The listing root and `archive`, `tags`, `page` prefixes are reserved for generated views.

Frontmatter `date` accepts a Gregorian `YYYY-MM-DD` or RFC3339 timestamp with an explicit timezone. Exact timestamps are preserved in output `publishedAt`; output `date` retains the publisher's calendar day. Without a declared date or filename date, a post stays undated. Farm does not derive publication dates from build time or filesystem modification time.

Authors may be display-name strings or objects containing `name`, optional `url` and `image`. Tags may be strings or explicit `{id, label}` objects; their IDs are normalized to portable Unicode URL segments. Unknown object fields, invalid dates/URLs and duplicate or conflicting tag IDs fail with source-file and field diagnostics.

Rust exports public `blogIndexes` with per-collection ordered post keys, tag/year groups and an undated group. Ordering uses actual publication instants, then stable public URLs; previous/next never crosses locale/collection boundaries.

Rust also generates the collection listing and archive, plus a tag index and individual tag pages when tags exist. Empty blogs have readable listing/archive pages without invented empty tag routes. These views have compiled bodies and actual public routes, distinct from the `index` article. `blogLabels` on each blog collection can override `title`, `archive`, `tags`, `undated` and `noPosts`; omitted labels keep the defaults, so translations do not depend on a fixed language table.

Generated pages expose public `blogView` data with their view kind, ordered post keys and optional tag. Their virtual source text/digest stays in the private plan; child builds validate that source and continue checking every real article file. Alternates are connected after real views exist, so an unavailable translated tag points to an existing target blog home with `missing: true`. Feeds, site-wide styling, SEO and production-site migration retain their later implementation gates.

The default `BlogPage` displays the given publication date, authors, resolved author images and actual tag links. Its `time` element preserves an explicit `publishedAt` value; a date-only article keeps its date, and an undated article receives no invented timestamp. Author order and links come from Rust metadata. Blog navigation uses the existing collection listing, archive and tag index, with titles from the generated pages. Empty blogs omit nonexistent tag navigation. Metadata and navigation sit outside the compiled article body, preserving its single heading and generated lists. Both ordinary links with JavaScript disabled and same-document client navigation are covered, including explicit translated fallback metadata and localized tag targets.

## Build input and output verification

Farm Docs uses the native `@farmfe/plugin-docs-snapshot` plugin in its build workflow. Rust records original module bytes before loading, registered watch dependencies, and the final resources exported by each browser/server compilation. Newly discovered watch inputs trigger a bounded observed retry; ordinary source changes and compilation errors fail the build.

After rendering, a separate native verification pass checks the content discovery set, source bytes, compiled resources, template, rendered HTML and public report before the output transaction installs the candidate. A detected change preserves the previous successful site. Private receipts stay in the owned staging directory and are removed with it. Node records file writes and coordinates compiler/worker lifetimes; it does not discover or validate compilation dependencies.

Persistent cache candidates are checked against original file and watch input hashes, including immutable modules. Changes with unchanged modification times still invalidate stale candidates. This does not monitor unregistered file reads by arbitrary plugins, transient changes restored before observation, or provide a filesystem lock against external writes between final verification and installation.

Compiler cache files are generated artifacts, not watch sources. Rust plugins can query `context.cache_manager.disk_cache_directory()` for the directory owned by the already-created cache manager; it includes the disk format, namespace and mode, and returns `None` for memory storage. Farm Docs rejects explicit inputs from this directory and excludes it when observing an ancestor directory. Directory exclusions compare resolved path identities, including Windows path spelling differences, while explicitly watched dependency directories still observe their own contents.

## Code copying and admonitions

The default theme maps Markdown `pre` elements to its public `CodeBlock` component. Rust supplies the original code in `data-code`, including its terminal newline, and generates syntax tokens, titles, line numbers and highlighted lines. The React component displays that tree and copies `data-code` directly; it does not parse or highlight code again. A plain `pre` without `data-code` remains a plain `pre`.

Copy buttons appear after mounting. A successful clipboard write reports `Copied.` through a status region; unavailable or rejected clipboard access reports `Could not copy. Try again.` The button stays disabled while its write is pending, and leaving the page discards its eventual feedback. Without JavaScript the code and title remain readable, with no inactive copy button. Long code scrolls inside its block on narrow screens.

Fenced metadata accepts a JSON quoted title, for example `title="hello world.ts"`, or an unquoted title without spaces. Quoted titles preserve JSON escapes; named and numeric character references in titles decode once after parsing, so entity quotes cannot change the metadata grammar. `showLineNumbers` and ranges such as `{1,3-4}` control Rust-generated decorations and never enter the copied text. Unknown languages retain plain code text. Admonitions retain their Rust-generated title and body, with a visible border in light and dark themes.

## Search

For a standalone search URL, add an ordinary React page and import `SearchPage` from `@farmfe/docs-theme-default`:

```tsx
import { useSite } from '@farmfe/docs/client';
import { SearchPage } from '@farmfe/docs-theme-default';

export const metadata = { title: 'Search', description: 'Search documentation' };
export default function Search() {
  return <><h1>Search</h1><SearchPage site={useSite()} /></>;
}
```

With `pages.dir` pointing at this page directory, Rust indexes `search.tsx` and creates its localized routes, metadata and language alternates. The component does not create routes or read a Docusaurus configuration. It renders a GET form during static generation; without JavaScript the page and form remain usable, while remote results require JavaScript. Initial hydration matches static HTML, then reads the URL's `q` parameter. Submitting a query preserves other query parameters and the fragment; refresh and Back/Forward restore the submitted query. Draft edits take effect on submission. Language links use the target Rust route and its own query, starting empty when that route has no `q`.

`SearchPage` and the modal share the same transport, cancellation, safe result links and retry behavior. With no provider the standalone page renders `Search is not configured.` and makes no remote requests. Its optional `labels` prop overrides `input`, `submit`, `idle`, `loading`, `error`, `empty`, `retry`, `results`, `unconfigured` and `count: (count: number) => string`; use `useLocale()` and your own translations to supply localized text.

Programmatic forms can call `navigate(to)` from `@farmfe/docs/client`. It accepts a relative or absolute HTTP(S) URL and uses the mounted Docs navigation queue, including resource loading, history ownership and failure recovery. When the runtime does not handle a destination it performs a document navigation. Call it from browser event handlers; rendering should continue to use ordinary links and forms.

Configure `search: { provider: 'algolia', options: { appId, apiKey, indexName } }` with a public search-only key. The default Navbar includes the named `Search` component from `@farmfe/docs-theme-default`; a custom navbar can render `<Search site={site} />`. Omitted search or `{ provider: 'none' }` renders no search entry and makes no remote search requests. JavaScript-disabled pages retain their content without an inactive search button.

After mounting, the search button opens a modal and focuses its input. Nonempty input sends a debounced, read-only request using the [Algolia single-index query API](https://www.algolia.com/doc/rest-api/search/search-single-index). Loading, no-results and failure states are announced, and failed requests can be retried. Requests time out after ten seconds; changing or clearing input, closing the modal, navigating or unmounting cancels the prior request. Late responses cannot overwrite a newer query.

Results support DocSearch `hierarchy.lvl0` through `lvl6` as well as ordinary `title`, `content` and `url` records. Result text is rendered as text, including snippets; remote highlighted HTML is not injected. Results require HTTP(S) URLs. Absolute results on the configured site origin become path/query/fragment links so registered pages keep normal Farm Docs client navigation in previews. External HTTP(S) results remain ordinary links. Search does not invent local routes for records that are missing from the current build.

Escape, the close button or backdrop closes the modal. Tab stays within it, and arrow keys move from the input through result links; Enter follows a focused result. Closing restores the prior scroll policy, while history navigation releases the modal before page focus moves. Controlled request tests cover these behaviors; they do not establish the availability, ACLs or contents of a particular hosted index. Site owners configure those with their search provider.

## SSG worker rendering

The Rust configuration protocol accepts `ssg: { concurrency: 4, timeoutMs: 30000 }`. Both values are integers: concurrency accepts 1 through 32, and timeout accepts 100 through 300000 milliseconds. Omitted fields use those deterministic defaults. Invalid ranges report `DOCS_CONFIG_SSG` with the field name; malformed types or unknown fields use the normal configuration parse diagnostic.

These settings enter the private build plan and its snapshot identity, rather than the public browser configuration. An old private plan without the required SSG fields must be prepared again. The workflow renders Rust-provided page tasks in a bounded worker pool. Each worker loads only the compiled server artifact and validates its snapshot. A page exceeding timeoutMs, throwing, or exiting fails the build with its route; the previously successful output remains intact. The pool waits for thread termination before staging cleanup or output installation. The provisional defaults still require measurement against the migrated website.

Worker initialization has a deadline of the greater of one second and timeoutMs. The per-page deadline covers rendering after dispatch; output writes are separately awaited before the worker receives another page. These limits do not set a timeout for filesystem writes or the entire native compilation. Build workflows accept runtime cancellation through the Core build API and CLI SIGINT/SIGTERM handlers. Cancellation stops subsequent stages and terminates SSR workers, awaiting in-flight output writes before cleanup. An active native compilation is awaited because it currently has no cancellation API; synchronous profiling also delays JavaScript signal callbacks. Cancellation before installation preserves or restores the previous output. Once installation begins, commit or recovery completes without interruption, so a completed commit may return success despite a concurrent cancellation. Builds hold a native operating-system lock for the entire site, including recovery and cleanup; a competing build fails with DOCS_OUTPUT_LOCKED before starting compilation. The private .farm-docs-locks directory contains a persistent site.lock and output transaction journals. Lock files retain their identity and are never deleted by the framework; ownership ends on release or process exit. On the next build, matching journals restore a verified previous output or recognize a completed installation before compiling new source. Unknown paths, conflicting output or damaged ownership records stop recovery and retain the backup. Cleanup receipts allow recovery even if a previous recursive cleanup already removed its staging marker. This covers process-exit recovery; it does not promise power-loss durability or gap-free reads between directory renames.

Rust rejects private tooling directories (.farm-docs-locks, .farm-docs and .git) as content or static inputs, copied assets and ordinary module dependencies. Component imports, aliases and raw imports are checked in the actual module graph; a private dependency fails with DOCS_PRIVATE_MODULE. Transaction records are never production content inputs. Keep both Farm Docs metadata directories out of version control.


## Experimental SEO configuration and plan

The Rust configuration contract adds an optional site setting, `seo: { fallbackIndexing: 'noindex' }`. This belongs to the first argument of `defineDocsConfig`, rather than Farm's compilation settings. The policy accepts `noindex` (the default) or `index`. Unknown fields, duplicate policy keys, non-object input and other values are rejected. Explicit indexing of fallback content does not make it a real translation.

The normalized public configuration contains the policy. Rust prepares a required per-page SEO registry with canonical URLs, robots directives and actual same-version language alternates; fallback copies are excluded from hreflang. The registry is part of the prepared snapshot and is recomputed during verification. Existing private plans must be prepared again after rebuilding the matching workspace native packages. Explicit canonical trailing-slash exceptions remain supported independently of the navigation URL policy.

The independent Rust SEO plugin consumes this verified plan. Browser builds publish sitemap.xml, robots.txt and an rss.xml under each blog collection's routeBase; server builds do not duplicate these resources. Generated files participate in the resource manifest and precommit verification. Conflicts with static files or other resources fail instead of overwriting existing content. The sitemap uses canonical URLs and actual language alternates, omitting fallback copies by default. Explicit fallback indexing does not duplicate RSS articles.

RSS includes actual articles in the Rust blog index order, with titles, canonical links, optional descriptions and tags. Undated posts omit pubDate. Date-only values use UTC midnight; timestamps normally normalize to UTC. If that conversion would leave years 1 through 9999, the valid original offset is retained so publication dates remain representable. Builds never invent the current time for missing publication metadata.

Enable Atom explicitly with `seo: { atom: { updated: '2026-10-18T00:00:00Z', author: 'Documentation contributors' } }`. Rust emits `atom.xml` under each actual blog collection's route base. `updated` is a declared feed update time with a timezone; it is also used for undated entries, which retain their content identity and omit `published`. Dated entries use their actual publication instant, with date-only values interpreted at UTC midnight. The feed uses the latest configured or emitted entry instant. Original valid timezone offsets remain representable at years 0001 and 9999. The explicit feed author supplies inherited author information; actual article authors remain attached to their entries. Titles, summaries and tags are XML text. Omitting `seo.atom` disables Atom while retaining RSS.

For browser search discovery, add `openSearch: { pageId: 'search', shortName: 'Project Docs', description: 'Search documentation' }` beside the Algolia `provider` and `options`. The logical `pageId` must identify exactly one real independent page in every declared locale. Rust uses those pages' actual routes to generate the `?q={searchTerms}` templates; it does not assume a `/search` path. Output is `opensearch.xml` for the default locale and `<locale>/opensearch.xml` for the others. `basePath` affects the public URLs rather than their output-directory paths. Short names allow up to 16 Unicode characters, and descriptions up to 1024. Empty text, invalid dates, non-object values, duplicate or unknown fields, missing target pages and output collisions are rejected.

Rust supplies safe `discoveryLinks` entries (`rel`, `type`, `title`, `href`) for RSS, enabled Atom and OpenSearch. SSR, successful client navigation, development updates and localized missing pages consume the same links. Client updates replace only links marked `data-farm-docs-discovery`; custom unrelated links remain. JavaScript-disabled HTML retains the discovery links as well as canonical URLs, robots directives and language alternates. The complete website migration and distribution acceptance remain pending; see the local Farm Docs progress record for the exact verified scope.

## Physical NotFound output

Rust prepares one auxiliary output for every declared locale: the default locale uses `404.html`, and other locales use `<locale>/404.html`. For a site mounted at `/guide/` with default `en`, the `zh` item has the URL scope `/guide/zh/` and physical output `zh/404.html`. `basePath` is a URL prefix and is not added to physical output paths. These auxiliary pages are excluded from content routes, page counts, language alternates and the sitemap. They use `noindex,follow` without canonical or hreflang links. Static files, compiled assets and content outputs cannot occupy their reserved paths, including case and ancestor collisions.

Configure plain-text labels in the Farm site configuration:

```ts
notFound: {
  labels: {
    zh: {
      title: '未找到页面',
      description: '请求的页面不存在。',
      homeLabel: '回到首页',
    },
  },
},
```

The locale must be declared in `locales.items`; unknown locales and blank labels produce `DOCS_CONFIG_NOT_FOUND` with a field path. Omitted label fields and locales receive English defaults. Rust uses `title` as the heading and combines it with the site title for the document title. The home target is an existing same-locale page, respecting configured collection priority and the default version. A locale with no actual pages falls back to a real default home.

The dedicated SSR worker task passes the locale to the compiled server's `renderNotFound(locale)` export, with the normal limits, cancellation and awaited cleanup. Private plan version 2 contains the ordered locale list. Private writes version 3 records each locale, output path and HTML digest; public report version 2 lists `{ locale, routeBase, outputPath }` separately from `pages`. Rust checks both the manifest's safe auxiliary table and the report against the plan. Missing, altered, reordered or conflicting records fail verification even when receipt hashes are recalculated. Old protocol versions are rejected; regenerate plans and rebuild with matching workspace native packages. Preserve or remove an old version's output explicitly before selecting a clean build destination.

NotFound hydration records an explicit locale without a content key and verifies it against Rust's URL selection. It renders the actual theme component without loading a fictitious page. Body and Head selection use the same navigation target, including before history is committed. History navigation restores the localized missing view; ordinary pages restore their canonical metadata. Development keeps the locale through content HMR; configuration label changes publish a new candidate, while invalid labels retain the successful generation until repaired.

The generated application graphs expose `notFoundPages`, `getNotFound(path)` and `renderNotFound(locale)`. The safe table excludes private output paths and raw Head HTML. Selection matches exact mount roots or slash boundaries, so `/zhish/` cannot match `/zh/`; Unicode paths are normalized as URLs. Unknown render locales are rejected. The rendering entry defaults to the site's default locale when called without an argument.

## Experimental static preview

`farm preview` consumes an existing successful Farm Docs build. It does not compile content. The report's `preview.basePath` comes from the verified Rust site configuration and is checked again before output installation; older artifacts without this field require a matching rebuild. The public manifest and report determine the physical output members the service may read.

Programmatic consumers can use the Node-only entry:

```ts
import { startDocsPreview } from '@farmfe/docs/preview';

const preview = await startDocsPreview({
  root: process.cwd(),
  server: { preview: { host: '127.0.0.1', port: 1911, strictPort: true } }
});
console.log(preview.url);
// When finished, await preview.close(). Repeated close calls share one promise.
```

The optional second argument accepts a runtime AbortSignal. Pre-cancelled startup does not open a listener; cancellation during startup is handled after the pending network operation finishes, then closes the service. CLI options such as host, port, strictPort and outDir use Farm's resolved preview settings. The server reuses Core's HTTP lifecycle, headers, HTTPS, CORS, proxy and compression facilities.

Static GET and HEAD requests use exact physical files or a directory's actual index.html. Missing pages and resources use physical404 HTML with status404; `404.html` itself is an existing physical file and returns200. No unknown URL becomes the homepage. Paths are decoded once; malformed escapes, encoded separators and traversal are rejected. Symlink traversal, private tooling files and unregistered extra output files are not served. This preview implementation is undergoing product HTTP/browser/lifecycle acceptance; the local progress record distinguishes its source/unit checks from completed native acceptance.

## Migrating saved tab choices

The public `Tabs` component from `@farmfe/docs/mdx` accepts an optional presentation setting for migrating an older numeric selection:

```mdx
<Tabs groupId="tools" storageMigration={{ key: 'legacy.tools', values: ['npm', 'yarn', 'pnpm', 'bun'] }}>
  <TabItem value="npm" label="npm">npm content</TabItem>
  <TabItem value="yarn" label="yarn">yarn content</TabItem>
</Tabs>
```

`values` describes the old index order. After hydration, a group without a current saved value may read a nonnegative integer from the legacy key and select the corresponding enabled item. Existing current storage always takes precedence. Invalid, out-of-range or unavailable choices use the normal default. The legacy key is retained. When legacy storage is readable but writing is denied, the choice still applies in memory; denied storage does not prevent keyboard or click interaction. Server rendering keeps every panel readable without JavaScript.

For Rust-compiled command panels, a custom module selected through `snippets.rendererModule` can wrap `CommandTabs` from `@farmfe/docs/snippets-runtime` and pass the same `storageMigration` presentation prop. This prop belongs to the renderer, rather than the author-facing static `CommandTabs` marker. Custom renderers may reorder already compiled panels for a site's presentation; parsing, command values and token generation remain in Rust.

## Composing a site layout

`theme.components` can select a React module exporting any of the public theme slots. A custom `Layout` receives `ThemeProps`, including the Rust-resolved page, home target, alternates, sidebars and composed `components`. Render the supplied page slot and pass through its props and children; do not discover routes again in the layout.

`@farmfe/docs-theme-default` also exports `MobileNavigation` and `ColorModeControl` for custom layouts. `MobileNavigation` accepts `ThemeProps` plus an optional `navigation?: ReactNode` slot for primary links and language/version controls. It keeps the default dialog keyboard/focus handling, scroll lock, close-on-navigation behavior, sidebar and table of contents. Pass site-specific navigation explicitly; the dialog does not infer it. `ColorModeControl` uses the current Docs provider. Both remain safe for server rendering; CSS and a client media subscription can choose a site's responsive breakpoint. Sites importing this package directly should declare it as a dependency.

## Localizing default admonition titles

A docs or blog collection can set `admonitionLabels` to supply default titles in its language:

```json
{
  "id": "zh",
  "dir": "content/zh",
  "locale": "zh",
  "routeBase": "/zh/docs/",
  "admonitionLabels": { "note": "备注", "tip": "提示", "warning": "注意" }
}
```

The supported keys are `note`, `tip`, `info`, `warning`, `danger`, `caution`, `important` and `success`. Missing keys retain their type name. An empty string hides that type's implicit title. Explicit author titles, including an explicitly empty title, always take precedence. Labels are plain text and remain private compilation inputs. Rust applies the final collection's labels to fallback content, independently of the source-language page; full blog listings reuse the original article's compiled labels.
