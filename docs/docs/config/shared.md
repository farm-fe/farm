# Shared Options

Configure shared options for Both Farm's DevServer and Compiler. Example:

```ts
import { defineConfig } from "farm";

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
  envDir?: string;
  envPrefix?: string | string[];
  /** Files under this dir will always be treated as static assets. serve it in dev, and copy it to output.path when build */
  publicDir?: string;
  /** js plugin(which is a javascript object) and rust plugin(which is string refer to a .farm file or a package) */
  plugins?: (RustPlugin | JsPlugin | JsPlugin[])[];
  /** vite plugins */
  vitePlugins?: (object | (() => { vitePlugin: any; filters: string[] }))[];
  // /** config related to compilation */
  // compilation?: Pick<InternalConfig, AvailableUserConfigKeys>;
  // /** config related to dev server */
  // server?: UserServerConfig;
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
import { defineConfig } from 'farm';
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

Configure Farm Plugins. See [Using Farm Plugins](/docs/using-plugins#farm-compilation-plugins)

## vitePlugins
- **default**: `[]`

Configure Vite/Rollup/Unplugin plugins. See [Using Vite Plugins](/docs/using-plugins#using-viterollupunplugin-plugins-in-farm)