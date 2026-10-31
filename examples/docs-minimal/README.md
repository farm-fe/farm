# Farm Docs minimal consumer

This workspace example uses only the public `@farmfe/docs` configuration API and the default theme. Content discovery, Markdown/MDX compilation, application generation and resource manifests run in Rust. Node orchestrates Farm builds and React server rendering.

After the workspace core and native Docs, React and SSR CSS plugins have been built, build `packages/docs` and `packages/docs-theme-default` with their `npm run build` scripts. Link dependencies with `pnpm install`, then run `npm run build` in this directory. The output is `build/`, mounted at `/guide/`.

`docs/index.mdx` imports an interactive React component relative to its source file. `docs/getting-started.md` uses ordinary Markdown semantics. Both pages have static HTML; the counter becomes interactive after hydration.

The framework is under development. Development/preview workflows, full navigation, multilingual/version indexing and release packaging are tracked separately in `docs/farm-docs`; this example currently exercises the production build path.
