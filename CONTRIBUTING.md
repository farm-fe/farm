# Contributing to Farm

Thank you for your interest in contributing to Farm!. Before submitting your contribution, please make sure to take a moment and read through the following guidelines:


## Code of Conduct

All contributors are expected to follow our [Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).


## Bug reports

As farm is currently in the process of rapid development iteration, some unexpected problems may be encountered in the process of development.

We can't fix what we don't know about, so please report problems and unexpected behavior.

You can open a new issue by following [new-issues](https://github.com/farm-fe/farm/issues/new/choose) and choosing one of the issue templates.

## Feature requests

Please feel free to open an issue using the [feature request template](https://github.com/farm-fe/farm/issues/new/choose).


## Pull Request Guidelines

- Fork the Farm repository into your own GitHub account.
- Please adhere to the code style that you see around the location you are working on.
- Setup Your Development Environment.
- Checkout a topic branch from a base branch, e.g. `main`.
- Run `cargo test` and make sure that it passes.
- If you've changed some packages And prepare for an updated version, you should output `npx changset` in the root directory.


## Development Environment Setup

### Dependencies

- Install Rust using [rustup](https://www.rust-lang.org/tools/install).

- [Node.js](https://nodejs.org) **version 16+**

- [Pnpm](https://pnpm.io) **version 7.28.0+**

### Setup Other Dependencies

- Install [protoc](https://grpc.io/docs/protoc-installation/) for building `sass-embedded`.

**TIP:** When you run `pnpm bootstrap` and you use mac or linux systems, farm will automatically install protoc for you system

## Start running

Farm development is very simple. You only need to execute pnpm bootstrap in the root directory for development.

```bash
$ pnpm bootstrap # install the dependencies of the project
```

- use `pnpm bootstrap` to install dependencies and build core packages with series of initialization operations.

- Work with examples (open a new terminal): `cd examples/react && pnpm start`, report an issue if the example does not start normally.  

- If `examples/react` project runs successfully, the development environment has been configured successfully

- If you changed Rust code in `crates`, run `npm run build:rs` under `packages/core` again to get the latest binary.


## Pull Request Preface Tip

Farm is divided into two parts: the `JavaScript side` and the `Rust side`:

- **the JavaScript side**: 
  see code in the `packages` directory. contains core (dev server, file watcher, and compiler wrapper), CLI, runtime, and runtime plugins (module system, HMR).
- **the Rust side**: 
  see code in the `crates` and `rust-plugins` directory. contains core (compilation context, plugin drivers, etc.), compiler (compile process, HMR update, etc.), and plugins.


## 