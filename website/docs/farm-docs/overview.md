# Farm Docs

**Prerelease documentation.** Farm Docs is being prepared for the v2 release.

Farm Docs builds documentation websites with Farm. Its Rust implementation handles Markdown/MDX parsing, content validation, route planning and compilation extensions. React renders the default theme and browser interface; Node orchestrates development, static generation and preview.

This is the documentation framework used by the Farm v2 website. It is separate from Farm's general application build configuration and does not require the Docusaurus runtime.

## What it provides

- Markdown/MDX document collections and generated navigation.
- Explicit locale and version collections, with generated alternate page targets.
- React theme slots, document sidebars, table of contents, code blocks and responsive navigation.
- A registered Farm workflow for development, static generation and local preview.
- Native validation for duplicate routes, missing references and unavailable content.

The default theme is `@farmfe/docs-theme-default`. Site branding and page-specific layout belong in the site; reusable reading components belong in the theme. Compilation extensions belong in native Rust plugins.

## Continue

Read the [usage guide](./usage.md) for project structure, configuration, themes, build and preview. Farm Docs does not automatically publish packages or deploy a website.

## Release notes

The framework is under development in the v2 workspace. A successful local workspace build does not establish that matching official packages and every platform binary have been published. Before external installation, verify the release's package versions, optional native dependencies and isolated consumer results. Do not substitute an older package or silently compile the website's dependencies from source during deployment.

## Configuration and customization reference

See the complete [configuration reference](./configuration.md) for fields, defaults and validation rules, and [theme customization](./theme-customization.md) for CSS tokens, React slots, aliases and Rust code rendering.
