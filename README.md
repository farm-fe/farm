<p align="center">
  <a href="https://npmjs.com/package/@farmfe/core"><img src="https://img.shields.io/npm/v/@farmfe/core.svg" alt="npm package"></a>
  <a href="https://nodejs.org/en/about/releases/"><img src="https://img.shields.io/node/v/@farmfe/core.svg" alt="node compatibility"></a>
  <a href="https://github.com/farm-fe/farm/actions/workflows/rust-test.yaml"><img src="https://github.com/farm-fe/farm/actions/workflows/rust-test.yaml/badge.svg" alt="build status"></a>
</p>
<br/>

# Farm

> Super fast web build tool written in rust. yet another performant alternative besides webpack/vite

|                     | Webpack | Vite  | Farm  | Compare                                       |
| ------------------- | ------- | ----- | ----- | --------------------------------------------- |
| **cold start**      | 853ms   | 276ms | 67ms  | Farm is faster: **12x webpack**，**4x vite**  |
| **HMR**             | 43ms    | 23ms  | 2ms   | Farm is faster: **20x webpack**，**10x vite** |
| **onload**          | 83ms    | 310ms | 57ms  | Farm is faster: **5x vite**                   |
| **accessible time** | 936ms   | 586ms | 124ms | Farm is faster: **8x webpack**，**5x vite**   |

> Test Repo：https://github.com/farm-fe/performance-compare
>
> Test Machine（Linux Mint 21.1 Cinnamon， 11th Gen Intel© Core™ i5-11400 @ 2.60GHz × 6， 15.5 GiB）

<br />

**Features**:

- 🔥 **Super Fast**: Written in Rust, start a react / vue(incoming) project in milliseconds, perform a HMR update within 10ms for the most situations.
- 🧰 **Fully Pluggable**: Everything inside Farm is powered by plugins, achieve anything you want by creating a plugin. Support both rust plugins and js plugins.
- ⚙️ **Native Web Assets Compiling Supported**: Support compiling JS/TS/JSX/TSX, css, html natively.
- **Lazy Compilation**: Dynamic imported resources are compiled only when they are requested.
- **Partial Bundling**: Bundle your project into a few reasonable bundles, speed up the resources loading without losing the caching granularity.
- **Consistency**: What you see in development will be exactly the same as what you've got in production.
- **Compatibility**: Support both legacy(es5) and modern browsers.

<br/>

> **Note**:
>
> - See [RFC-001](https://github.com/farm-fe/rfcs/blob/main/rfcs/001-core-architecture/rfc.md#motivation) for design motivation and principle.
> - **This project is still under development. Contributions are welcome**.
>
> This project is built on the SWC Project, using swc for html/css/js/tsx/ts/jsx parsing, transforming, optimizing and codegen.

<br/>

## Getting Started

Create a new Farm Project

```sh
npx @farmfe/cli@latest create
```

Start the project:

```sh
cd farm-react && npm i && npm start
```

Refer to [Documentation](https://farm-fe.github.io) to learn more about Farm.

## Contribution
Farm is divided into two parts: the `js side` and the `rust side`:
* **the js side**: see code in `packages` dir, contains core(dev server, file watcher and compiler wrapper), cli， runtime and runtime plugins(module system, hmr)
* **the rust side**: see code in `crates` dir, contains core(compilation context, plugin drivers...), compiler(compile process, HMR update...) and plugins.

Steps to deveplop Farm:
1. Install Rust Toolchain(If you are new to Rust, search `Rustup Book`) and node 16 or above.
2. Install dependencies with `pnpm i`.
3. Build the compiler binary: `cd packages/core && npm run build:rs`
4. Build packages(open a new terminal): `cd packages/cli && npm start`
5. Work with examples(open a new terminal): `cd examples/react && npm start`, report an issue if the example do not start normally.
6. If you changed Rust code, run `npm run build:rs` under `packages/core` again to get the newest binary.
