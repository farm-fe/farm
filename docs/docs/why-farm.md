---
sidebar_position: 2
---

# Why Farm?

## What is Farm?
Farm is an extremely fast Rust-based web build tool, like `webpack` and `vite`, but **`much faster`**. Farm resolves, loads, and transforms all of your `assets(js/jsx/ts/tsx, css/sass/less, html, static assets, json, etc)`, and bundle them into a set of `deployable files`. Farm is an extremely fast build tool that helps you build faster `web/nodejs` apps.

## Why Farm?

As web projects scale, build performance has been their major bottleneck. For a huge project compiling with webpack may cost 10 or more minutes and an HMR update may cost 10s or more, heavily reducing development efficiency.

Then, tools like Vite came out. It uses native ESM and is unbundled for source files in dev mode, pre-bundles dependencies using esbuild, which makes the dev server launch and the HMR very fast.

But Unbundled is not perfect, there are still big problems when comes to a large project:
* **The huge number of module requests**: For a large project, there may be thousands of modules that need to be loaded. Using the native module system to load thousands of modules will make the browser get stuck or even cause it to crash.
* **Inconsistency between Dev and Production**: Native modules cannot be used in production for most situations, due to compatibility and request number issues. So Unbundled tools choose to bundle in production. This brings inconsistency, when there are production bugs caused by this inconsistency, it's really hard to debug and really painful. Vite uses esbuild in dev and using rollup in production, which makes the inconsistency worse.
* **Inflexible Chunk Splitting**: Configuration for Chunk Splitting is not flexible enough.
* Vite is so fast in dev because of esbuild, which is written in go. Go takes advantage of the native platform and is much faster than JS.

So I think we just need a fast, powerful, consistent web bundler, which can solve the problems above and fast, then I designed and implemented Farm.

And Farm is not just a normal bundler re-written in Rust, it has a lot of powerful and progressive designs:

## Farm Design Philosophy

* **Performance first**: Everything will be written in Rust for as long as we can; only several parts which are not the performance bottleneck will be written in JS.
* **Consistency first**: Make sure that development and production are exactly the same by default. What you see in development will be the same as what you get in production.
* **Partial Bundling**: The bundling goal of Farm is not to bundle everything together, but to limit the request numbers of resources. Farm will bundle your project into 20-30 small resources according to the dependency relation and resource size, to get the best resource loading performance without losing caching granularity.
* **First class citizen support of all web assets**: Farm won't need to transform everything to Javascript any more, it treats anything as first class citizen, assets like `html`, `js/jsx/ts/tsx`, `css/scss`, `png/svg/...` are all basic modules supported by Farm, more assets can be supported by plugins.
* **Compatibility**: Farm will work with both legacy (ES5) and modern browsers.
* **Rollup style plugin system and vite/rollup compatible js-plugins**: Easy to create your own plugins and easy to migrate your plugins/projects from rollup/vite. Support both Rust and JS plugins.

Farm's goal is to be the real next generation build tool, inherit all advantages from existing tools, and to be fast, powerful, consistent, and provide the best development experience for web developers.
