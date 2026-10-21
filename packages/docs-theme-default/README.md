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
