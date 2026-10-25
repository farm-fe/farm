# Overview

Farm provides official Rust compilation plugins as the primary extension path for its native compiler, alongside JavaScript plugins for existing ecosystem integrations. Choose by required features and measured workload, not an assumed universal speed ratio.

:::tip
Refer to [Using Plugins](/docs/using-plugins) for how to use plugins in Farm.
:::

## Rust native capability map

This is a source inventory for the v2 candidate, not a statement that every package is published, feature-complete or release-accepted. The package pages below define supported options and limitations. Install versions and platform native packages must match your compiler candidate.

| Family | Implementations in this repository | Scope and status |
| --- | --- | --- |
| Compiler foundation | `crates/compiler`, `crates/plugin_*` | Native module graph, scripts, CSS/HTML/JSON, resolution, bundling and optimization; part of the candidate compiler. |
| Frameworks | `rust-plugins/react`, `vue`, `vue-jsx` | React JSX/refresh and Vue compilation. Vue delegates SFC compilation to `fervid`; check each page's feature limits rather than assuming all Vue/Vite behavior. Vue JSX is source-available experimental integration. |
| Styling | `rust-plugins/sass`, `tailwindcss` | Sass/SCSS and Rust Tailwind generation. Check supported syntax/configuration; this is not a claim of complete compatibility with every JavaScript ecosystem plugin. |
| Import and component automation | `rust-plugins/auto-import`, `react-components`, `modular-import` | API/component discovery and import transformations; source available, with per-plugin candidate coverage. |
| Assets and data | `rust-plugins/svgr`, `icons`, `image`, `url`, `yaml`, `dsv`, `virtual` | SVG/components, icons, image/URL handling, YAML/CSV/TSV and virtual modules; use each plugin's documented import contract. |
| Output and runtime integration | `rust-plugins/worker`, `wasm`, `compress`, `dts`, `strip`, `replace-dirname` | Worker/Wasm integration, compression, declarations and script transformations; candidate scope varies. |
| Documentation | `rust-plugins/docs`, `docs-snippets`, `docs-seo`, `docs-snapshot`, `syntax-highlight`, `ssr-css` | Native content/compiler plugins with `packages/docs` Node/React orchestration and UI. These source-available packages are not proof of a published Farm Docs release. |

Some plugins adapt or port existing open-source work. Their upstream licenses, attribution and notices remain applicable; a Rust implementation does not remove those obligations. Consult each package's license/notice files and upstream provenance before redistribution. This inventory is not a completed license audit.

## Rust Plugins

* **[`@farmfe/plugin-react`](./react)**：Support React `jsx` and `react-refresh`.
* **[`@farmfe/plugin-vue`](./vue)**：Compile Vue 3 Single-File Components via the `fervid` Rust compiler.
* **[`@farmfe/plugin-vue-jsx`](./vue-jsx)**: Experimental Rust Vue JSX integration; check the documented syntax and HMR limits.
* **[`@farmfe/plugin-sass`](./sass)**：Support compiling `sass/scss` files.
* **[`@farmfe/plugin-auto-import`](./auto-import)**：Inject imports from presets and scanned local exports.
* **[`@farmfe/plugin-tailwindcss`](./tailwindcss)**：Rust-based TailwindCSS integration.
* **[`@farmfe/plugin-svgr`](./svgr)**：Transform SVG files into React components.
* **[`@farmfe/plugin-wasm`](./wasm)**：Support WebAssembly and `wasm-pack` generated modules.
* **[`@farmfe/plugin-worker`](./worker)**：Support web workers via constructors and worker query suffixes.
* **[`@farmfe/plugin-url`](./url)**：Import files as data URIs or emitted asset URLs.
* **[`@farmfe/plugin-dts`](./dts)**：Emit `.d.ts` files for matched TypeScript modules.
* **[`@farmfe/plugin-icons`](./icons)**：Use Iconify icons as on-demand framework components.
* **[`@farmfe/plugin-image`](./image)**：Import image files as base64 data URIs or DOM `Image` exports.
* **[`@farmfe/plugin-mdx`](./mdx)**：Compile `.md` and `.mdx` files to JSX.
* **[`@farmfe/plugin-compress`](./compress)**：Compress generated assets with Brotli, Gzip, Deflate, or DeflateRaw.
* **[`@farmfe/plugin-modular-import`](./modular-import)**：Transform UI-library named imports into per-component imports.
* **[`@farmfe/plugin-replace-dirname`](./replace-dirname)**：Replace `__dirname`, `__filename`, and `import.meta.url` in script modules.
* **[`@farmfe/plugin-strip`](./strip)**：A Farm rust plugin to remove `debugger` statements and functions like `assert.equal` and `console.log` from your code.
* **[`@farmfe/plugin-dsv`](./dsv)**：A Farm plugin which converts `.csv` and `.tsv` files into JavaScript modules.
* **[`@farmfe/plugin-yaml`](./yaml)**：A Farm plugin which Converts YAML files to ES6 modules.
* **[`@farmfe/plugin-virtual`](./virtual)**：A rust plugin for farm to easily use virtual module.
* **[`@farmfe/plugin-react-components`](./react-components)**：On-demand components auto importing for React.

## Js Plugins

* **[`@farmfe/js-plugin-postcss`](./js-postcss)**：Support `postcss` in your project.
* **[`@farmfe/js-plugin-babel`](./js-babel)**：Run Babel on matched Farm modules.
* **[`@farmfe/js-plugin-copy`](./js-copy)**：Copy files and globs at the end of a Farm build.
* **[`@farmfe/js-plugin-less`](./js-less)**：Support compiling `less` files.
* **[`@farmfe/js-plugin-svgr`](./js-svgr)**：Support compiling `svg` files.
* **[`@farmfe/js-plugin-dts`](./js-dts)**：Support compiling `*.d.ts` files.
* **[`@farmfe/js-plugin-sass`](./js-sass)**：Support compiling `sass/scss` files.
* **[`@farmfe/js-plugin-tailwindcss`](./js-tailwindcss)**：Support TailwindCSS integration.
* **[`@farmfe/js-plugin-visualizer`](./js-visualizer)**：Visualize your bundle size and composition.
* **[`@farmfe/js-plugin-electron`](./js-electron)**：Support building Electron applications.
* **[`@farmfe/js-plugin-qiankun`](./js-qiankun)**：Expose Farm apps as qiankun micro-apps.
* **[`@farmfe/js-plugin-react-compiler`](./js-react-compiler)**：Run React Compiler through Babel for JSX/TSX modules.
* **[`@farmfe/js-plugin-vuetify`](./js-vuetify)**：Support Vuetify auto-imports and style resolution.

## Community Plugins

If official plugins doesn't meet your needs, you can try [Community Plugins](../community-plugins)

And of course check out [awesome-farm](https://github.com/farm-fe/awesome-farm) - you can also submit a PR to list your plugins there.
