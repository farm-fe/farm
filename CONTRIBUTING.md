# Farm Contributing Guide

Thank you for your contribution to farm. Before submitting your contribution, please make sure to take a moment and read through the following guidelines:

### Pull Request Preface Tip~

Farm is divided into two parts: the `JavaScript side` and the `Rust side`:

- **the JavaScript side**: see code in the `packages` directory, contains core (dev server, file watcher, and compiler wrapper), CLI, runtime, and runtime plugins (module system, HMR).
- **the Rust side**: see code in the `crates` and `rust-plugins` directory, contains core (compilation context, plugin drivers, etc.), compiler (compile process, HMR update, etc.), and plugins.

### Pull Request Guidelines

- Fork the Farm repository into your own GitHub account and clone your repository.
- Checkout a topic branch from a base branch, e.g. `main`.
- Make sure tests pass!
- If you've changed some packages, you should output npx changset in the root directory.

### Development Environment Setup

##### Dependencies

- Install Rust using [rustup](https://www.rust-lang.org/tools/install).

- [Node.js](https://nodejs.org) **version 16+**

- [PNPM](https://pnpm.io) **version 7.28.0+**

##### Setup Other Dependencies

- Install [protoc](https://grpc.io/docs/protoc-installation/) for building `sass-embedded`.

TIP: When you run `pnpm bootstrap` and you use mac or linux systems, farm will automatically install protoc for you

##### After cloning the repo, run:

```bash
$ pnpm bootstrap # install the dependencies of the project
```

- use `pnpm bootstrap` to install dependencies and build core packages.

- Work with examples (open a new terminal): `cd examples/react && npm start`, report an issue if the example does not start normally.

- If you changed Rust code in `crates`, run `npm run build:rs` under `packages/core` again to get the latest binary.
