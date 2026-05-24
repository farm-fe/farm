---
sidebar_position: 1
---

# Farm v2 is Ready

Farm v2 is the stable release of Farm's next-generation Rust build toolchain. It brings the v2 compiler, runtime, CLI, JavaScript plugins, and Rust plugin ecosystem out of beta and makes the `latest` release line ready for production use.

## Highlights

- **All in Rust where performance matters**: Farm v2 keeps the compiler pipeline, bundling, transformations, and the official Rust plugin foundation in Rust for fast startup, rebuilds, and production builds.
- **Rich Rust plugin ecosystem**: official Rust plugins cover React, Sass, DSV, YAML, virtual modules, strip, wasm, workers, URL/assets, declarations, compression, icons, and more. Farm v2 also includes Rust-based Tailwind CSS support through `@farmfe/plugin-tailwindcss`.
- **Library bundling**: build libraries with ESM, CJS, UMD, IIFE, SystemJS, or AMD output. Farm supports single-bundle, multiple-bundle, and bundle-less modes for packages and component libraries.
- **Production optimization by default**: production builds enable tree shaking, minification, partial bundling, cache-aware resource splitting, polyfills, CSS/HTML optimization, and stable hashing for efficient deploys.
- **Vite-compatible development experience**: keep the familiar config and plugin model while using Farm's Rust-first compiler and consistent dev/prod bundling model.
- **Stable v2 release line**: Farm packages and Rust crates now use stable v2 versions instead of beta prerelease versions.

## All in Rust

Farm v2 continues the original Farm design goal: move performance-critical work out of JavaScript and into Rust. The compiler core, module graph, resolver, script/CSS/HTML handling, partial bundler, minifier integration, runtime generation, persistent cache, and Rust plugin ABI are all designed around Rust performance and safety.

The Rust plugin ecosystem is now a first-class part of Farm v2. Use official Rust plugins when you need fast framework transforms, asset handling, declaration generation, compression, Tailwind CSS, or other build-time extensions.

## Library Bundling

Farm v2 can build packages as libraries, not just browser applications. Set `output.targetEnv` to `library`, choose one or more output formats, and select the bundling strategy that matches your package:

- `single-bundle` for one-file package outputs.
- `multiple-bundle` for multiple entries with shared chunks.
- `bundle-less` for component libraries that should preserve source structure and maximize consumer-side tree shaking.

See [Library Bundling](/docs/features/library) for configuration details.

## Production Optimization

Farm v2 production builds are optimized out of the box:

- Tree shaking removes unused ESM exports.
- Minification compresses and mangles JavaScript and optimizes CSS/HTML.
- Partial bundling balances request count, caching granularity, and resource size.
- Persistent cache accelerates repeated builds.
- Output hashing and asset handling are ready for long-term browser caching.

These optimizations are enabled through Farm's Rust compiler pipeline, so development and production behavior remain consistent while production output stays efficient.

## Upgrade

Install Farm v2 from the stable release channel:

```bash
pnpm create farm@latest
pnpm add @farmfe/core@latest @farmfe/cli@latest
```

For existing projects, upgrade Farm packages and official plugins to their v2 stable versions, then remove any `beta` or `nightly` dist-tags from your Farm dependencies.
