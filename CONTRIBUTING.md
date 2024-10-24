# Contribution Guide

Thank you for your interest in contributing to Farm!. Before submitting your contribution, please make sure to take a moment and read through the following guidelines.

## Code of Conduct

All contributors are expected to follow our [Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## Bug reports

As farm is currently in the process of rapid development iteration, some unexpected problems may be encountered in the process of development.

We can't fix what we don't know about, so please report problems and unexpected behavior.

You can open a new issue by following [new-issues](https://github.com/farm-fe/farm/issues/new/choose) and choosing one of the issue templates.

## Feature requests

Please feel free to open an issue using the [feature request template](https://github.com/farm-fe/farm/issues/new/choose).

## Pull Request Guidelines

- Please adhere to the code style that you see around the location you are working on.
- Setup Your Development Environment.
- Checkout a topic branch from a base branch, e.g. `main` .
- Run `cargo test` and make sure that it passes.
- If you've changed some packages And prepare for an updated version, you should output `npx changeset` in the root directory. we should try our best to keep releasing the `patch version`. If there are no major changes, please choose to update the `patch version`.

- When you are done with your work, verify that it works locally with `pnpm run ready`

## Setup

- Fork and clone the repo.

- Create a branch for your PR with `git checkout -b your-branch-name`.

- To keep `main` branch pointing to remote repository and make pull requests from branches on your fork. To do this, run:

```bash
  git remote add upstream https://github.com/farm-fe/farm.git
  git fetch upstream
  git branch --set-upstream-to=upstream/main main
```

## Development Environment Setup

### Dependencies

- Install Rust using [rustup](https://www.rust-lang.org/tools/install).

- [Node.js](https://nodejs.org) **version 16+**

- [Pnpm](https://pnpm.io) **version 8+**

### IDE

We recommend that you use vscode for development and recommend two necessary plugins that you need to install

- `rust-analyzer` support rust language
- `biome` farm use biome to detect and format the code

You can install them in the extension

### Setup Other Dependencies

- Install [protoc](https://grpc.io/docs/protoc-installation/) for building `sass-embedded`.

**TIP:** When you run `pnpm bootstrap` and you use mac or linux systems, farm will automatically install protoc for you system

## Start running

Farm development is very simple. You only need to execute `pnpm bootstrap` in the root directory for development.

```bash
$ pnpm bootstrap
```

- use `pnpm bootstrap` to install dependencies and build core packages with series of initialization operations.

- Work with examples (open a new terminal): `cd examples/react && pnpm start`, report an issue if the example does not start normally.

- If `examples/react` project runs successfully, the development environment has been configured successfully

- If you changed Rust code in `crates`, run `npm run build:rs` under `packages/core` again to get the latest binary.

When you are developing node side code, the root directory executes pnpm start to debug the code in real time, and when you are developing rust side code, the root directory executes pnpm start:rs to debug the code in real time.

```ts
// node side
pnpm start

// rust side
pnpm start:rs
```

## Testing

We also need to test two parts, a set of `Rust` tests and a set of `Node` tests. Make sure all the tests pass before you submit the code.

### Rust Testing

- Input `cargo test` in the root directory will run all the test cases.

```sh
# root path or crates path
cargo test
```

If you want to update snapshot, you can use `INSTA_UPDATE=always cargo test`

```sh
INSTA_UPDATE=always cargo test
```

### Node Testing

- Input `pnpm test` in the root directory to run all test cases based on `vitest`.

```sh
# root path
pnpm test
```

## Quickly create plugins through scaffold

If you want to develop a plugin for farm, farm provides a scaffolding to help you quickly create a plugin, which you can create with the following command.
You can go to the `cd packages/ cli` directory, run `npm link` or global installation `@ farmfe/ cli` to use this CLI, after the installation is complete, You can create a plugin through `farm plugin create`.
Farm supports the creation of rust and js plugins.

```bash
$ farm plugin create <plugin-name> # create a plugin support js or rust
```

## Pull Request Preface Tip

Farm is divided into two parts: the `JavaScript side` and the `Rust side`:

- **the JavaScript side**:
  see code in the `packages` directory. contains core (dev server, file watcher, and compiler wrapper), CLI, runtime, and runtime plugins (module system, HMR).
- **the Rust side**:
  see code in the `crates` and `rust-plugins` directory. contains core (compilation context, plugin drivers, etc.), compiler (compile process, HMR update, etc.), and plugins.
