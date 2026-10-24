---
sidebar_position: 2
---

# Why Farm?

## What is Farm?

Farm is a Rust native web build tool. Its compiler and built-in compilation plugins handle the module graph, transformation, bundling and optimization in Rust. Official Rust plugins extend this pipeline for frameworks, styling, assets and documentation.

Node.js provides configuration, CLI and development-server orchestration. JavaScript plugins, React rendering and application runtimes remain part of the stack where needed. Vite-compatible plugin adapters provide a migration path; they are not the definition of Farm’s architecture.

[Explore the official Rust plugin families](/docs/plugins/official-plugins/overview). The v2 implementation is a candidate until its packages and release checks are complete.

## Why Farm?

Farm focuses on the native compilation path: one Rust compiler owns module processing and bundling, while official Rust plugins extend it for frameworks, styling, assets and documentation. This reduces the need to cross the JavaScript/native boundary for those supported compilation tasks. It does not make Node.js, React SSR or application code disappear.

Persistent caching, lazy compilation and partial bundling provide different ways to manage project scale. Their benefits must be measured with the same workload, cache state and deployment requirements. No universal startup, HMR or build-time multiplier is implied.

JavaScript plugins and Vite-compatible adapters remain available when they match a project’s needs. Check supported hooks and run development, hot-update and production tests when migrating; compatibility is not a promise of identical behavior for every external plugin.

## Farm Design Philosophy

* **Rust native compilation**: keep compiler work and native compilation extensions in Rust. Node.js still orchestrates configuration, development services and JavaScript integration; documentation rendering can use React SSR.
* **Consistency first**: Make sure that development and production are exactly the same by default. What you see in development will be the same as what you get in production.
* **Partial Bundling**: The bundling goal of Farm is not to bundle everything together, but to limit the request numbers of resources. Farm will bundle your project into 20-30 small resources according to the dependency relation and resource size, to get the best resource loading performance without losing caching granularity.
* **First class citizen support of all web assets**: Farm won't need to transform everything to Javascript any more, it treats anything as first class citizen, assets like `html`, `js/jsx/ts/tsx`, `css/scss`, `png/svg/...` are all basic modules supported by Farm, more assets can be supported by plugins.
* **Compatibility**: Farm will work with both legacy (ES5) and modern browsers.
* **Rollup style plugin system and vite/rollup compatible js-plugins**: Easy to create your own plugins and easy to migrate your plugins/projects from rollup/vite. Support both Rust and JS plugins.

Farm's goal is to be the real next generation build tool, inherit all advantages from existing tools, and to be fast, powerful, consistent, and provide the best development experience for web developers.
