# Farm CLI Reference

Source docs: `website/docs/cli/cli-api.md`

---

## Commands

```bash
farm [root]           # Start dev server (default command)
farm build            # Production build
farm watch            # Watch mode (equivalent to build --watch)
farm preview          # Preview production build locally
farm clean [path]     # Clear persistent cache
farm plugin [command] # Plugin management
```

---

## Common Flags

Available on most commands:

```bash
-c, --config <file>   # Use custom config file
-m, --mode <mode>     # Set env mode (development / production / custom)
--port <port>         # Dev server port
--host <host>         # Dev server host
--open                # Open browser on server start
--hmr                 # Enable hot module replacement
--cors                # Enable CORS
--strictPort          # Exit with error if port is in use
--base <path>         # Public base path
-l, --lazy            # Enable lazy compilation
--clearScreen         # Allow/disable clearing screen on recompile
-h, --help            # Display help
-v, --version         # Display version
```

---

## `farm start` / `farm [root]`

Start the dev server and compile in development mode.

```bash
farm                          # start from current directory
farm ./my-app                 # start from a subdirectory
farm -c farm.custom.config.ts # use a custom config file
farm --port 3000 --open       # custom port + open browser
farm --lazy                   # enable lazy compilation
```

---

## `farm build`

Build production artifacts to `dist/` (or `output.path`).

```bash
farm build
farm build --watch            # rebuild on file changes
farm build --targetEnv node   # build for Node.js
farm build --format cjs       # output format
farm build --sourcemap        # emit source maps
farm build --treeShaking      # enable tree shaking
farm build --minify           # enable minification
farm build -o ./out           # custom output directory
farm build -i ./src/main.ts   # custom entry file
```

---

## `farm preview`

Locally preview the production build. Run `farm build` first.

```bash
farm preview
farm preview --port 4173
```

---

## `farm watch`

Rebuild on file changes (without a dev server).

```bash
farm watch
farm watch -c farm.node.config.ts
```

---

## `farm clean`

Clear Farm's persistent cache directory (`node_modules/.farm/cache`).

```bash
farm clean
farm clean ./custom-cache-path
```

---

## `farm plugin`

Manage Farm plugins (scaffold, list, etc.).

```bash
farm plugin --help
```
