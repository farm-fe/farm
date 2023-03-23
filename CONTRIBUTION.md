# Contribution

Farm is divided into two parts: the `JavaScript side` and the `Rust side`:

- **the JavaScript side**: see code in the `packages` directory, contains core (dev server, file watcher, and compiler wrapper), CLI, runtime, and runtime plugins (module system, HMR).
- **the Rust side**: see code in the `crates` directory, contains core (compilation context, plugin drivers, etc.), compiler (compile process, HMR update, etc.), and plugins.

Steps to develop Farm:

1. Install Rust Toolchain (If you are new to Rust, search for "Rustup Book") and Node.js 16 or above.
2. Install dependencies and build core packages with `pnpm bootstrap`.
3. Work with examples (open a new terminal): `cd examples/react && npm start`, report an issue if the example does not start normally.
4. If you changed Rust code in `crates`, run `npm run build:rs` under `packages/core` again to get the latest binary.
