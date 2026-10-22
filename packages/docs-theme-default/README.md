# Farm Docs default theme

The reusable React theme for `@farmfe/docs`. It renders Rust-generated page metadata, sidebars, blog views and navigation without depending on Docusaurus configuration or runtime.

The public stylesheet is available as `@farmfe/docs-theme-default/style.css`. Farm Docs includes this stylesheet through its workflow; custom themes can override styles through `theme.customCss`.

## Override slots

Set `theme.components` to a site-local module. Export any subset of `Layout`, `DocPage`, `BlogPage`, `Navbar`, `Sidebar`, `Toc`, `Footer` and `NotFound`; omitted slots retain their defaults. A default export overrides `Layout`.

```tsx
import type { ThemeProps } from '@farmfe/docs/theme';

export function Footer({ site }: ThemeProps) {
  return <footer>{site.title}</footer>;
}
```

A custom `Layout` should render slots from `props.components` and forward complete props. This preserves page content, generated navigation and optional `listingPosts` for full blog listings. `NotFound` uses its separate `NotFoundProps` type. Export a `components` object for MDX content component overrides.

Read generated locale/version alternates and navigation targets from public page data. Do not derive them by concatenating locale or version URL segments. Keep server rendering and initial browser output consistent; initialize browser-only effects after mounting.

Use the `@farmfe/docs` peer version matching this theme. Package distribution, platform binaries and isolated consumer verification are tracked by the Farm Docs release checks; this README does not assert those checks have passed.


## Reading presentation

The default layout wraps its slots in `.farm-docs-theme`. A site replacing the
layout should retain that class to use the shared reading styles. The theme owns
the document columns, typography, controls, code frames, tabs, admonitions,
responsive navigation and collapsible mobile table of contents. Site CSS should
set brand tokens and style its own pages rather than duplicate those components.

Public tokens include `--farm-docs-font-family`, `--farm-docs-font-monospace`,
`--farm-docs-link`, `--farm-docs-accent-soft` and `--farm-docs-accent-border`.

`Sidebar` preserves the order and targets in `data.sidebars`. Its optional
`expandedCategories` prop names sections that start expanded; ancestors of the
active document also open. Configure document order through collection sidebar
JSON, not by discovering or rewriting routes in a React component.

`DocPage` accepts optional `homeLabel`, `versionLabel` and `tocLabel` presentation
props. Site wrappers can translate these labels while retaining fallback notices,
article content and previous/next navigation. The shared theme contains no Farm
website category names or version policy.
