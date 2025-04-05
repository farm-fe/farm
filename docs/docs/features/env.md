# Environment Variables and Modes

`Farm` distinguishes between `development` and `production` environments through `process.env.NODE_ENV`.

In different environments, environment variables are replaced statically, so use static constants to represent environment variables instead of dynamic expressions.

## `.env` file

`Farm` uses `dotenv` to load your additional environment variables, such as `.env` files. By default `.env` file are loaded from [`root`](/docs/config/shared#root), you can use [`envDir`](#envdir) to customize.

```js
// .env
FARM_APP_SECRET=secret
Farm_APP_PASSWORD=password
APP_VERSION=1.0.0
```

`Farm` loads the file `.env` via dotenv, and loads it into `process.env` and finally injects it into define.

:::danger
In order to ensure the security of the client, preventing the environment variables in the current system from being exposed to the client `Farm` will only identify some important environment variables that start with `FARM_`、`VITE_` (In order to better compatible with vite and its ecological environment).
:::

`Farm` expands environment variables through dotenv-expand. For development only envs use `.env.development`, for production only envs use `.env.production`, for custom mode passed by `--mode <stage>`, load from `.env.<stage>` file.

* If you want to customize the directory to load `.env` file, you can configure [`envDir`](#envdir).
* If you want to customize the prefix of env variables which are injected to [`define`](/docs/config/compilation-options#define), you can configure [`envPrefix`](#envprefix).


## envPrefix

- **default value**: `FARM_`、`VITE_`

Customize the prefix of the `env` variable by configuring `envPrefix`. Env variables start with `envPrefix` will be injected into define automatically. For example, in the `.env` file:

```js
// .env
FARM_CUSTOM_VERSION=1.0.0
APP_VERSION=0.1.0
```
Then `FARM_CUSTOM_VERSION` will be injected, but not `APP_VERSION`, in your business code:

```tsx
export function MyComp() {
  const farmCustomVersion = FARM_CUSTOM_VERSION;
  return <div>Farm Custom Version: {farmCustomVersion}</div>
}
```
`FARM_CUSTOM_VERSION` will be replaced by `'1.0.0'` automatically.



## envDir
- **default value**: `<root>`

The directory to load [`env file`](#env-file). By default Farm load `env file` from root.

```ts
export defineConfig({
  envDir: './env'
})
```

For above config example, Farm will load `.env`, `.env.development`, etc from `<root>/env` dir.
