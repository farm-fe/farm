---
sidebar_position: 2
---

# Why Farm?

As the web project scales, building performance has been the major bottleneck, for a huge project, compiling with webpack may cost 10min or more, a hmr update may cost 10s or more, heavily reduced the efficiency.

Then some tools like vite comes out, it uses native ESM and is unbundled for source files in dev mode, pre-bundle dependencies using esbuild, which makes the dev server launch and the HMR very fast.

But Unbundled is not perfect, there are still big problem when comes for a large project:
* **The huge numbers of module requests**: For a large project, there may be thousands of modules that should be loaded, using native module system to load thousands of modules will make the browser get stuck or even crashed.
* **Inconsistency between Dev and Production**: Native module can not be used in production for most situations, For the compatibility and request numbers. So Unbundled tools choose to bundle in production. This brings inconsistency, when there are production bugs caused by this inconsistency, it's really hard to debug and really painful. And vite is using esbuild in dev and using rollup in production, which enlarged the inconsistency.
* **Inflexible Chunk Splitting**: Configuration for Chunk Splitting is not flexible enough.
* And Vite is so fast in dev because of esbuild, which is written in go. Go takes advantages of native platform and much faster than Js.

So I think we just need a fast, powerful, consistent web bundler, which can solve the problems above and fast, then I designed and implemented Farm.

And Farm is not just a normal bundler re-written in Rust, it has a lot of powerful and progressive designs:

## Farm Design Philosophy

* **Performance first**: Everything will be written in Rust as long as we can, only several parts which is not the performance bottleneck will be written in JS
* **Consistence first**: Make sure that the development and production exactly the same by default, what you see in development will be the same as what you got in production.
* **Partial Bundling**: The bundling goal of Farm is not to bundle everything together, but to limit the request numbers of resources. Farm will bundle your project into 20-30 small resources according to the dependency relation and resource size, to get the best resources loading performance without losing caching granularity.
* **First class citizen support of all web assets**: Farm won't need to transform everything to Javascript any more, it treats anything as first class citizen, assets like `html`, `js/jsx/ts/tsx`, `css/scss`, `png/svg/...` are all basic modules supported by Farm, more assets can be supported by plugins.
* **Compatibility**: Farm will work with both legacy(ES5) and modern browser.
* **Rollup style plugin system and vite/rollup compatible js-plugins**: Easy to create your own plugins and easy to migrate your plugins/projects from rollup/vite. Support both Rust and JS plugins.

Farm's goal is to be the real next generation build tool, inherit all advantages from existing tools, and to be fast, powerful, consistent, and providing best development experience for web developers.
