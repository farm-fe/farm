<p align="center">
  <a href="https://npmjs.com/package/@farmfe/core"><img src="https://img.shields.io/npm/v/@farmfe/core.svg" alt="npm package"></a>
  <a href="https://nodejs.org/en/about/releases/"><img src="https://img.shields.io/node/v/@farmfe/core.svg" alt="node compatibility"></a>
  <a href="https://github.com/farm-fe/farm/actions/workflows/rust-test.yaml"><img src="https://github.com/farm-fe/farm/actions/workflows/rust-test.yaml/badge.svg" alt="build status"></a>
</p>
<br/>

# Farm

> Super fast web build engine written in rust. yet another performant alternative besides webpack/vite

<br />
Started in 68ms and updated in 1ms for a demo react project as below.

![img](./assets/performance.png)

**Features**:
* 🔥 **Super Fast**: Start a react / vue(incoming) project in milliseconds.
* ⚡ **"1ms" HMR**: Finish a HMR within 10ms for the most situations.
* 🧰 **Fully Pluggable**: Support both rust plugins and js plugins.
* ⚙️ **Native Web Assets Compiling Supported**: Support support compiling JS/TS/JSX/TSX, css, html natively.

<br/>

> **Note**:
>
> - See [RFC-001](https://github.com/farm-fe/rfcs/blob/main/rfcs/001-core-architecture/rfc.md#motivation) for design motivation and principle.
> - **This project is still under development. Contributions are welcome**.
>
> This project is built on SWC Project, using swc for html/css/js/tsx/ts/jsx parsing, transforming, optimizing and codegen.

<br/>

## Getting Started
Install Farm Cli:
```sh
npm install -g @farmfe/cli
```

We provided a experience react project for now. Using `farm create` to create a new project. Using `farm start` to start the project.

```sh
farm create && cd farm-react && npm i && npm start
```
