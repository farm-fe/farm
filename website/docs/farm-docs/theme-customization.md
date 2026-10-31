# Customize a Farm Docs theme

**Prerelease APIs.** Use matching verified packages; this page does not advertise an already published install command. Start with [configuration](./configuration.md) and keep the default theme unless you need to change its presentation. Rust owns content/routes/compilation; React owns rendering and interaction. A theme does not replace the content compiler.

## Small complete example

```text
my-docs/
  farm.config.ts
  docs/index.md
  src/theme.tsx
  src/custom.css
```

`farm.config.ts`:

```ts
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig({
  title: 'Example Docs',
  url: 'https://docs.example.com',
  docs: { collections: [{ id: 'main', dir: 'docs', routeBase: '/docs/' }] },
  theme: {
    components: 'src/theme.tsx',
    customCss: ['src/custom.css'],
    colorMode: 'system',
    navbar: [{ label: 'Guide', to: '/docs/' }]
  },
  build: { highlightRepresentation: 'static-html' }
}, { compilation: { output: { path: 'build' } } });
```

`docs/index.md`:

```md
# Welcome

This page keeps the default document layout and a custom footer.
```

`src/theme.tsx`:

```tsx
import type { ThemeProps } from '@farmfe/docs/theme';
import { Link } from '@farmfe/docs/client';

export function Footer({ site }: ThemeProps) {
  return <footer><span>{site.title}</span><Link to="/docs/">Guide</Link></footer>;
}
```

Only `Footer` is replaced. The default `Layout`, document page, sidebar, TOC and 404 remain. Put CSS from the next section in `src/custom.css`, then run the installed CLI's `farm build` and `farm preview`. Normal lifecycle/optional dependencies and matching native packages remain prerequisites. The framework automatically imports default theme/highlight styles before custom styles; do not add a second browser highlighter.

## CSS tokens and scope

Current default-theme variables include:

| Tokens | Purpose |
| --- | --- |
| `--farm-docs-bg`, `--farm-docs-text`, `--farm-docs-muted` | Reading background and text |
| `--farm-docs-link`, `--farm-docs-border` | Accent/links and separators |
| `--farm-docs-panel`, `--farm-docs-hover` | Surface and interactive background |
| `--farm-docs-code-bg` | Code container background |
| `--farm-docs-font-family`, `--farm-docs-font-monospace` | UI/body and code font stacks |

These are CSS variables used by the shipped stylesheet, not `SiteConfig` fields or a versioned token JSON API. Check the actual installed stylesheet when upgrading. Keep changes scoped to the Docs shell rather than changing every `nav`, `button` or `pre` in a mixed application.

```css
html:not([data-theme="dark"]) .farm-docs-theme {
  --farm-docs-bg: #faf9fc;
  --farm-docs-text: #252131;
  --farm-docs-link: #7041a8;
  --farm-docs-border: #ded7e8;
}
html[data-theme="dark"] .farm-docs-theme {
  --farm-docs-bg: #19171f;
  --farm-docs-text: #efebf6;
  --farm-docs-link: #d4b2ff;
  --farm-docs-border: #40364f;
}
.farm-docs-theme article { line-height: 1.75; }
```

The selectors intentionally match the theme's specificity. Inspect computed colors in both modes; changing `:root` alone may lose to variables defined on the theme shell. Retain visible keyboard focus, readable contrast, code overflow within its container and a mobile layout without page-wide horizontal scrolling.

## React slots and content components

`theme.components` loads one site-relative module. It can export:

| Export | Props / role |
| --- | --- |
| `Layout` | `ThemeProps`; outer composition |
| `DocPage`, `BlogPage` | `ThemeProps`; document/article rendering |
| `Navbar`, `Sidebar`, `Toc`, `Footer` | `ThemeProps`; individual presentation regions |
| `NotFound` | `NotFoundProps`; generated locale-specific 404 data |
| `default` | Fallback override for `Layout` only |
| `components` | A content component map merged over the default map |

`ThemeProps` contains `site`, `page`, `children`, `data` (`home`, `pages`, `sidebars`, `blogIndexes`), `components` and optional `listingPosts`. `NotFoundProps` has `site`, `data` including `notFound`, and `components`; it is not an article `ThemeProps` with a fabricated page. Import these types from `@farmfe/docs/theme`.

Omitted slots keep defaults. Overrides must be valid React components; the resolver also recognizes React memo/forwardRef/lazy component markers. There is no automatic filesystem swizzle discovery.

When wrapping a layout, forward the whole contract so nested slot overrides continue to work:

```tsx
import { Layout as DefaultLayout } from '@farmfe/docs-theme-default';
import type { ThemeProps } from '@farmfe/docs/theme';

export function Layout(props: ThemeProps) {
  return <DefaultLayout {...props} />;
}
```

A wholly custom layout should render the intended slots from `props.components` and forward props/children. Keep `.farm-docs-theme` if you reuse its CSS. Replacing `Layout` with only `{children}` can accidentally remove navigation, search, document actions and 404 behavior.

For a content element, preserve its IDs, attributes and children:

```tsx
import type { ComponentPropsWithoutRef } from 'react';

function Heading(props: ComponentPropsWithoutRef<'h2'>) {
  return <h2 {...props} className={`example-heading ${props.className ?? ''}`} />;
}
export const components = { h2: Heading };
```

This changes rendered presentation, not parsed heading IDs/TOC/Markdown export. A custom MDX component still needs a real supported content/import contract; a React component map does not teach the AI exporter arbitrary dynamic semantics.

## Navigation, sidebar sections and search

Simple links come from `theme.navbar`. For custom dropdowns, icons and section-aware sidebars, replace `Navbar`/`Sidebar` and use public default-theme exports such as `NavigationMenu`, `NavigationIconLink`, `LanguageIcon`, `sidebarSection` and `filterSidebarSection` according to their exported TypeScript props. These helpers do not add menu-object fields to site JSON. The Farm website's section/prefix data is site code, not hard-coded framework categories.

Use `page.alternates`, generated page targets and `data.sidebars`. Do not invent translated/versioned routes by replacing URL prefixes. Use `Link`/`navigate` from `@farmfe/docs/client` for internal navigation and preserve normal external-link behavior. Custom menus must support pointer, click, keyboard and mobile touch.

The default theme exports `SearchProvider`, `Search` and `SearchProps`. A site can wrap `DefaultLayout` in `SearchProvider.Provider` with a custom React search component. This is the boundary for a DocSearch integration; credentials still come from the explicit public Algolia config, and the site must supply compatible DocSearch dependencies/styles. The framework does not create keys, crawlers or indices. The React extension is not a promise that the index contains v2 documents.

## Dark mode and code rendering

`useColorMode()` from `@farmfe/docs/client` exposes `colorMode`, `resolvedColorMode` and `setColorMode`. Valid choices are `system`, `light`, `dark`. The default initial choice is `theme.colorMode` (normally `system`); an available saved selection is restored by base path. CSS responds to `data-theme` and the system preference before/around hydration. Do browser-only work in effects; avoid accessing `window`, `localStorage` or `matchMedia` while rendering on the server.

Rust generates both light/dark code token colors. The compatibility default is VS Code Light/Dark. Select the optional Farm brand pair or custom scope colors through `site.code`; see the unified code palette configuration reference. Their `.farm-code-token` markup selects `--code-light`/`--code-dark` through the shipped highlight stylesheet. Changing `theme.colorMode` selects display mode; it does not run a JavaScript tokenizer or select an arbitrary code-theme package.

Explicit `CodeBlock`/`CommandTabs` markers come from `@farmfe/docs/snippets`, are compiled by the Rust snippets plugin, and share `site.code`. Presets include `farm-light`, `farm-dark`, `vscode-light`, `vscode-dark`, `inspired-github`, and `base16-ocean-dark`. Legacy `snippets.highlight.light` and `.dark` remain fallbacks for fields omitted from `site.code`; see the configuration reference for precedence. `snippets.rendererModule` can point to a presentation module exporting `CodeBlock` and `CommandTabs`. The site's `build.highlightRepresentation` controls the representation used by that workflow. Static source/markers must remain compilable; dynamic code expressions are not a runtime highlighting escape hatch.

`jsx` preserves compiler-generated token children for custom renderers that inspect them. `static-html` retains wrapper/interactivity while lowering generated escaped tokens to static HTML. If overriding `pre`/code components, forward the compiler's props and attributes; never stringify or discard its generated HTML/token representation. Use `jsx` when a renderer fundamentally requires individual token React children. Do not replace Rust highlighter work with Prism, Shiki or another browser runtime tokenizer.

## Aliases and migration boundaries

Use Farm's `compilation.resolve.alias` for site paths, for example `@site` resolving to the site root with `fileURLToPath(new URL('.', import.meta.url))`. Alias values are compiler paths; they are not public URLs. Ordered user aliases are preserved, but the workflow reserves its own Docs/client/runtime/theme module identities and gives framework aliases priority. Do not override them to inject another runtime or load two framework instances.

An alias resolves a path; it does not reproduce Docusaurus context, plugin lifecycle, theme internals or arbitrary MDX semantics. Replace old imports with actual `@farmfe/docs/client`, `/mdx`, `/snippets` APIs and keep small application adapters in the site. There is no package-level Docusaurus compatibility layer, swizzle command, arbitrary theme resolver, or automatic translation service.

Before accepting a custom theme, build and preview the exact candidate, test both modes, locale/version links, mobile navigation, search errors, code copy, tabs and keyboard focus. HTML/AI output should keep the same routes and content semantics. Production deployment and official package availability require their own verification.

## Farm website brand palette

The website explicitly selects `farm-light` / `farm-dark` and supplies its brand
colors through `code.tokens.light` and `code.tokens.dark`. This is a site
configuration recipe, not a change to the framework's default presets. Rust
applies the map during compilation to fences, code components and marked inline
signatures; no browser tokenizer is added.

| Token categories | Light | Dark |
| --- | --- | --- |
| `foreground`, `variable` | `#292435` | `#eeecf2` |
| `keyword`, `control`, `tag` | `#8f1a7f` | `#dc96cd` |
| `type`, `class`, `attribute` | `#526b87` | `#a8b9ce` |
| `function` | `#497774` | `#9fbebc` |
| `parameter` | `#5f596b` | `#c7bfd1` |
| `property` | `#676071` | `#b8b0c2` |
| `string`, `number`, `constant` | `#8a6e42` | `#c8b391` |
| `comment`, `punctuation` | `#746d7a` | `#9e99a4` |

Copy each category into the appropriate token map to reproduce the website
palette. Container backgrounds (`#faf7fb` / `#211d25`), spacing and rounded corners
remain CSS customization. The default stylesheet displays configured token
colors directly; it does not recolor them with a second CSS palette.

Official Mermaid diagrams are static, paired light/dark SVGs with editable
Mermaid sources. The optional authoring renderer is separate from `farm build`;
readers do not download a Mermaid runtime. Keep accessible captions and source
links when replacing a diagram's presentation.

## Explicit inline API signatures

Ordinary inline code, file paths and configuration names remain plain. For a literal API signature, opt into native compile-time token coloring with `data-language`; no browser highlighter runs. Use a fence for multiline code. Dynamic children and unsupported languages are rejected.

```html
<code data-language="ts">(moduleId: string) =&gt; void | Promise&lt;void&gt;</code>
```
