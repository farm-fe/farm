# Farm CLI

The Farm CLI starts the dev server, builds production output, watches builds, previews built assets, and cleans Farm's persistent cache.

Run this from a project that has `@farmfe/cli` installed:

```bash
npx farm --help
```

## Commands

| Command | Purpose |
| --- | --- |
| `farm [root]` | Start the development server. Aliases: `farm start`, `farm dev`. |
| `farm build [root]` | Compile the project for production. |
| `farm watch [root]` | Compile in development mode and rebuild when files change. |
| `farm preview [root]` | Serve an existing production build for local preview. |
| `farm clean [path]` | Remove Farm persistent-cache files. |

## Global options

These options are available to all commands unless noted by command-specific help:

| Option | Description |
| --- | --- |
| `-c, --config <file>` | Use a specific config file. Defaults to `farm.config.js`, `farm.config.ts`, `farm.config.mjs`, `farm.config.cjs`, `farm.config.mts`, or `farm.config.cts`. |
| `-m, --mode <mode>` | Set the config/env mode. |
| `--base <path>` | Set the public base path. |
| `-d, --debug [feat]` | Show debug logs, optionally scoped to a feature. |
| `--clearScreen` | Enable or disable terminal clear-screen behavior. Defaults to `true`. |
| `-h, --help` | Show help. |
| `-v, --version` | Show the installed `@farmfe/cli` and `@farmfe/core` versions. |

## `farm [root]`, `farm dev`, `farm start`

Start the Farm dev server and compile in development mode.

Common options:

| Option | Description |
| --- | --- |
| `-l, --lazy` | Enable lazy compilation. Defaults to `true`. |
| `--host <host>` | Specify the host. |
| `--port <port>` | Specify the port. |
| `--open` | Open the browser when the server starts. |
| `--hmr` | Enable hot module replacement. |
| `--cors` | Enable CORS. |
| `--strictPort` | Exit if the requested port is already in use. Defaults to `true`. |
| `--target <target>` | Set output target env: `node`, `browser`, or `library`. |
| `--format <format>` | Set output format: `esm` or `commonjs`. |
| `--sourcemap` | Emit source maps. |
| `--treeShaking` | Enable tree shaking. |
| `--minify` | Enable minification. |

## `farm build [root]`

Build production output.

| Option | Description |
| --- | --- |
| `-o, --outDir <dir>` | Output directory. |
| `-i, --input <file>` | Entry HTML/input file. |
| `--target <target>` | Set output target env: `node`, `browser`, or `library`. |
| `--format <format>` | Set output format: `esm` or `commonjs`. |
| `--sourcemap` | Emit source maps. |
| `--treeShaking` | Enable tree shaking. |
| `--minify` | Enable minification. |

## `farm watch [root]`

Build in development mode and rebuild on file changes. This command is usually used for Node/library-style outputs rather than serving an app.

| Option | Description |
| --- | --- |
| `-o, --outDir <dir>` | Output directory. |
| `-i, --input <file>` | Entry input file. |
| `--target <target>` | Set output target env: `node`, `browser`, or `library`. |
| `--format <format>` | Set output format: `esm` or `commonjs`. |
| `--sourcemap` | Emit source maps. |

## `farm preview [root]`

Serve already-built production assets. Run `farm build` first.

| Option | Description |
| --- | --- |
| `--host [host]` | Specify the host. |
| `--port <port>` | Specify the port. |
| `--open` | Open the browser when preview starts. |
| `--outDir <dir>` | Directory to serve. Defaults to `dist`. |
| `--strictPort` | Exit if the requested port is already in use. |

## `farm clean [path]`

Remove Farm persistent-cache files. By default it cleans the current project's resolved cache directory.

| Option | Description |
| --- | --- |
| `--recursive` | Search recursively for `node_modules` directories and clean Farm cache locations under them. |
