# Farm JavaScript API Reference

Source docs: `website/docs/api/javascript-api.mdx`

Package: `@farmfe/core`

```ts
import {
  start, build, watch, preview, clean,
  createCompiler, createDevServer, resolveConfig,
  loadEnv, Compiler, Server, logger,
} from '@farmfe/core';
```

---

## High-Level Functions

These functions accept an `InlineConfig` (same shape as `farm.config.ts` default export).

### `start(options)`

Start the development server.

```ts
await start({
  compilation: { input: { index: './index.html' } },
  server: { port: 3000, hmr: { path: '/__farm_hmr' } },
  plugins: ['@farmfe/plugin-react'],
});
```

### `build(options)`

Build for production.

```ts
await build({
  compilation: { output: { targetEnv: 'browser-es2017' } },
});
```

### `watch(options)`

Watch mode — rebuild on file changes without a dev server.

```ts
await watch(options);
```

### `preview(options)`

Preview production artifacts. Run `build()` first.

```ts
await preview(options);
```

### `clean(options)`

Clear the persistent cache.

```ts
await clean(options);
```

---

## Lower-Level API

For custom integration, compose the primitives manually:

```ts
const config   = await resolveConfig(inlineConfig);
const compiler = await createCompiler(config);
const server   = await createDevServer(compiler, config);
server.listen();
```

---

## `Compiler` Class

```ts
const compiler = new Compiler(resolvedConfig);
await compiler.compile();
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `compile` | `() => Promise<void>` | Async compilation |
| `compileSync` | `() => void` | Synchronous compilation |
| `update` | `(paths: string[], sync?, ignoreCheck?) => JsUpdateResult` | Incremental HMR update |
| `hasModule` | `(path: string) => boolean` | Check if path is in the module graph |
| `modules` | `() => Module[]` | All compiled modules |
| `resources` | `() => Resource[]` | All output resources |
| `resource` | `(path: string) => Buffer \| null` | Buffer for a single resource |
| `writeResourcesToDisk` | `() => void` | Write all resources to `output.path` |
| `removeOutputPathDir` | `() => void` | Delete the output directory |
| `outputPath` | `() => string` | Resolved output path |
| `traceDependencies` | `() => string[]` | All transitive dependencies of the entry |
| `resolvedModulePaths` | `(root: string) => string[]` | Module paths relative to `root` |
| `resolvedWatchPaths` | `() => string[]` | Currently watched paths |
| `getParentFiles` | `(resolvedPath: string) => string[]` | Importers of a module |
| `addExtraWatchFile` | `(root: string, paths: string[]) => void` | Add extra files to watch |
| `onUpdateFinish` | `(cb: () => any) => void` | Callback after each incremental update |

### `JsUpdateResult`

```ts
type JsUpdateResult = {
  success:  boolean;
  errors:   Error[];
  warnings: Error[];
};
```

### `Module`

```ts
interface Module {
  id:             string;
  moduleType:     string;
  moduleGroups:   string[];
  resourcePot?:   string;
  sideEffects:    boolean;
  sourceMapChain: string[];
  external:       boolean;
  immutable:      boolean;
}
```

---

## `Server` Class

```ts
const server = new Server({ compiler });
await server.createDevServer(options);
server.listen();
await server.close();

const compiler = server.getCompiler();
```

---

## `loadEnv`

Load environment variables from `.env` files programmatically.

```ts
type LoadEnvFunc = (
  mode: string,
  envDir: string,
  prefixes?: string | string[]
) => [env: Record<string, string>, existsEnvFiles: string[]];
```

```ts
const [env, files] = loadEnv(
  'development',       // mode
  process.cwd(),       // envDir
  ['FARM_', 'VITE_']   // prefixes (default)
);
```

Loads: `.env`, `.env.local`, `.env.[mode]`, `.env.[mode].local` from `envDir`.

---

## Error Handling Pattern

```ts
import { build, logger } from '@farmfe/core';

try {
  await build(options);
} catch (error) {
  logger.error(`Build failed:\n${error.stack}`);
  process.exit(1);
}
```
