# Environment variable

`Farm` distinguishes between development and production environments through `Farm` process.env.NODE\_ ENV`.

In different environments, environment variables are replaced statically, so use static constants to represent environment variables instead of dynamic expressions.

### `.env` file

`Farm` uses `dotenv` to load your additional environment variables, such as `.env` files.

```js
// .env
FARM_APP_SECRET=secret
Farm_APP_PASSWORD=password
APP_VERSION=1.0.0
```

`Farm` loads the file `.env` via dotenv, and loads it into `process.env` and finally injects it into define.

:::warning
In order to ensure the security of the client, preventing the environment variables in the current system from being exposed to the client `Farm` will only identify some important environment variables that start with `FARM_`、`VITE_` (In order to better compatible with vite and its ecological environment).
:::

`Farm` expands environment variables through dotenv-expand

If you want to customize the prefix of env variables, you can configure `envPrefix`.

### envPrefix

- **default value**: `FARM_`、`VITE_`

Customize the prefix of the `env` variable by configuring `envPrefix`.
