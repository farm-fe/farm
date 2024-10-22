# Farm CLI

The Farm CLI allows you to start, build, preview, and watch your application.

To get a list of cli available to Farm, run the following command inside your command

```json title="Terminal"
npx farm -h
```

The output look like this:

```json title="Terminal"
farm/0.5.11

Usage:
  $ farm [root]

Commands:
  [root]            Compile the project in dev mode and serve it with farm dev server
  build             compile the project in production mode
  watch             watch file change
  preview           compile the project in watch mode
  clean [path]      Clean up the cache built incrementally
  plugin [command]  Commands for manage plugins

For more info, run any command with the `--help` flag:
  $ farm --help
  $ farm build --help
  $ farm watch --help
  $ farm preview --help
  $ farm clean --help
  $ farm plugin --help

Options:
  -l, --lazy           lazyCompilation
  --host <host>        specify host
  --port <port>        specify port
  --open               open browser on server start
  --hmr                enable hot module replacement
  --cors               enable cors
  --strictPort         specified port is already in use, exit with error
  -c, --config <file>  use specified config file
  -m, --mode <mode>    set env mode
  --base <path>        public base path
  --clearScreen        allow/disable clear screen when logging
  -h, --help           Display this message
  -v, --version        Display version number
```

## Start

`farm start` The command is used to start the development server and compile the code in the development environment

```json title="Terminal"
Usage:
  $ farm [root]

Options:
  -l, --lazy           lazyCompilation
  --host <host>        specify host
  --port <port>        specify port
  --open               open browser on server start
  --hmr                enable hot module replacement
  --cors               enable cors
  --strictPort         specified port is already in use, exit with error
  -c, --config <file>  use specified config file
  -m, --mode <mode>    set env mode
  --base <path>        public base path
  --clearScreen        allow/disable clear screen when logging
```

## Build

`farm build` The command builds the products that can be used in the production environment in the default `dist` directory.

```json title="Terminal"
Usage:
  $ farm build

Options:
  -o, --outDir <dir>    output directory
  -i, --input <file>    input file path
  -w, --watch           watch file change
  --targetEnv <target>  transpile targetEnv node, browser
  --format <format>     transpile format esm, commonjs
  --sourcemap           output source maps for build
  --treeShaking         Eliminate useless code without side effects
  --minify              code compression at build time
  -c, --config <file>   use specified config file
  -m, --mode <mode>     set env mode
  --base <path>         public base path
  --clearScreen         allow/disable clear screen when logging
  -h, --help            Display this message
```

## Preview

`farm preview` the command for locally previewing the products built in your production environment, you need to execute farm build in advance to build the products in the production environment.

```json title="Terminal"
Usage:
  $ farm preview

Options:
  --open [url]          Whether to open the page in the browser at startup
  --port <port>         Set the port number for Server snooping
  --host <host>         Specify the host to listen to when Server starts
  -c --config <config>  Specify the profile path
  -h, --help            Show command help
```

## Watch

`farm watch` the command generally listen for file changes and rebuild in `node` environment

```json title="Terminal"

Usage:
  $ farm watch

Options:
  --format <format>    transpile format esm, commonjs
  -o, --outDir <dir>   output directory
  -i, --input <file>   input file path
  -c, --config <file>  use specified config file
  -m, --mode <mode>    set env mode
  --base <path>        public base path
  --clearScreen        allow/disable clear screen when logging
  -h, --help           Display this message
```

## Clean

`farm clean` Because the incremental build provided by `farm` generates the cache file locally, you may need to clean up the cache file under certain circumstances (unpredictable compilation errors)

```json title="Terminal"
Usage:
  $ farm clean [path]

Options:
  --recursive          Recursively search for node_modules directories and clean them
  -c, --config <file>  use specified config file
  -m, --mode <mode>    set env mode
  --base <path>        public base path
  --clearScreen        allow/disable clear screen when logging
  -h, --help           Display this message
```
