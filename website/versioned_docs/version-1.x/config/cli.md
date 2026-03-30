# CLI Options
## create
Create a new Farm project.

```bash
pnpm create farm
# or npm create farm
# or yarn create farm
# choose your favorite package manager
```

Other commands are provided by package `@farmfe/cli`:

## start
Start a dev server, compile the Farm project in development mode and watch file changes.

```bash
farm start
```

## build
Build a Farm project in production mode

```bash
farm build
```

## preview
Preview the result of `build` command.

```bash
farm build && farm preview
```

## watch
Watch is usually used to compile a library project, it works Like `start` command but it does not launch a dev server.

```bash
farm build
```