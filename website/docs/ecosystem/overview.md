# Farm ecosystem

Farm uses a Rust-native compiler and official Rust plugins. Its ecosystem also includes documentation tooling and reusable Rust libraries. Choose an entry by what you want to build.

## Build with Farm

Use the [official plugin documentation](/docs/plugins/official-plugins/overview) for supported compiler extensions and the [Rust plugin guide](/docs/plugins/writing-plugins/rust-plugin) to develop your own.

## Reuse Rust libraries

The [Rust crate directory](./rust-crates.md) groups verified libraries by purpose, identifies their source repositories and separates published releases from source-only or local candidates.

## Build documentation

[Farm Docs](/docs/farm-docs/overview) provides documentation collections, routing, themes and Markdown access for agents. Rust handles content parsing, validation and compilation extensions; Node orchestrates the workflow and React renders the interface and static pages.

Release status belongs to each project. A successful workspace build does not imply that every crate, npm package and native platform artifact has been published.
