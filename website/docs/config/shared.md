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