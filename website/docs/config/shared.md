# Shared Options

Configure shared options for Both Farm's DevServer and Compiler. Example:

```ts
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  // All dev server options are under server
  root: process.cwd(),
});
```

Type:
```ts
export interface UserConfig {
  /** current root of this project, default to current working directory */
  root?: string;
  clearScreen?: boolean;
  mode?: string;
  envDir?: string;
  envPrefix?: string | string[];
  /** Whether to enable file watching, or watch configuration options */
  watch?: boolean | WatchOptions;
  /** Files under this dir will always be treated as static assets. serve it in dev, and copy it to output.path when build */
  publicDir?: string;
  /** js plugin(which is a javascript object) and rust plugin(which is string refer to a .farm file or a package) */
  plugins?: (RustPlugin | JsPlugin | JsPlugin[] | undefined | null | false)[];
  /** vite plugins */
  vitePlugins?: (null | undefined | object | (() => { vitePlugin: any; filters: string[] }))[];
  /** config related to compilation */
  compilation?: CompilationConfig;
  /** config related to dev server */
  server?: UserServerConfig;
  /** Custom logger instance */
  customLogger?: Logger;
}
```
## root

- **default**: `process.cwd()`

Configure the root directory for project compilation. All relative paths are relative to `root` during compilation.

## clearScreen
- **default**: `true`

Whether to clear the screen when start to compile the project.

## envDir
- **default**: `<root>`

Configuring the directory to load `.env`, `.env.development`, `.env.production` files. By default it's the same as root.

```ts
import { defineConfig } from '@farmfe/core';
import { resolve } from 'path';
export default defineConfig({
  envPrefix: ['FARM_', 'CUSTOM_PREFIX_', 'NEW_'],
  envDir: resolve(process.cwd(), './env'),
});
```
In above example, will load `.env`, `.env.development`, `.env.production` files from `<root>/env` directory.

## envPrefix
- **default**: `['FARM_', 'VITE_']`

Env variables starts with `envPrefix` will be injected [`define`](/docs/config/compilation-options#define) automatically.

## publicDir
- **default**: `public`

Files under `publicDir` will always be treated as static assets. serve it in dev, and copy it to output.path when build.

For example, you can add static assets like font to `public` dir and using them as `/xxx.ttf`.

## plugins
- **default**: `[]`

Configure Farm Plugins. You can pass `null`, `undefined`, or `false` to conditionally disable a plugin. See [Using Farm Plugins](/docs/using-plugins#farm-compilation-plugins)

```ts
export default defineConfig({
  plugins: [
    pluginA(),
    process.env.ANALYZE && pluginAnalyze(), // conditionally enable
  ],
});
```

## vitePlugins
- **default**: `[]`

Configure Vite/Rollup/Unplugin plugins. See [Using Vite Plugins](/docs/using-plugins#using-viterollupunplugin-plugins-in-farm)

## mode
- **default**: `'development'` for `start`/`watch` commands, `'production'` for `build` commands

Configure the project mode. The value will be available as `process.env.NODE_ENV` at compile time and affects default optimization behavior. See also [`compilation.mode`](/docs/config/compilation-options#mode).

## watch
- **default**: `true` for `start`/`watch` commands, `false` for `build` commands

Whether to enable file watching, or detailed [chokidar](https://github.com/paulmillr/chokidar) watch options.

## customLogger
- **default**: `undefined`

Provide a custom logger instance to replace Farm's built-in logger.

## workflow

- **default**: `undefined`

Frameworks can provide a trusted local command runner to coordinate multiple Farm compilations and static rendering. A workflow is command orchestration, not a JavaScript compilation plugin. Compilation transforms still belong to compiler plugins.

```ts
export default defineConfig({
  workflow: {
    module: './site-workflow.mjs',
    options: { title: 'My documentation' },
  },
  compilation: { input: { index: './index.html' } },
});
```

The runner module exports `async function run({ command, config, options, signal })`. Farm dispatches `dev`, `build`, `watch`, or `preview` after resolving configuration and before creating a compiler or server. `config` is the resolved configuration with `workflow` removed, so a runner can use Farm APIs without dispatching itself recursively. Runner failures propagate to the caller.

`module` accepts a local relative path (relative to the project root), an absolute path, a file URL, or a locally resolvable package entry. Use only trusted project modules. Projects without `workflow` retain the standard Farm command behavior. Normal configuration resolution, including compilation input defaults, still applies before dispatch.


The optional runtime signal is passed through build(config, { signal }); it is separate from serialized configuration. Build workflows also receive cooperative SIGINT/SIGTERM cancellation, with listeners removed when the runner finishes. The runner must observe the signal at safe boundaries and finish any commit or recovery already in progress. Native compilation is not interrupted automatically.

Trusted workflow implementations can use OutputDirectoryLock from @farmfe/core for an OS-backed exclusive lock on an absolute regular file path. Keep the returned object alive until release(), and release it in finally. Competing acquisition throws DOCS_OUTPUT_LOCKED. The lock file must retain its identity: do not unlink or replace it while coordinating writers. This low-level bridge does not validate a framework's output paths or implement its recovery protocol.
