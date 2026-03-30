# 环境变量和模式

`Farm` 通过 `process.env.NODE_ENV` 来区分  `development` 环境和 `production` 环境。

在不同的环境中，环境变量会被静态替换，因此使用静态常量来表示环境变量，而不是使用动态表达式。

## `.env` 文件

`Farm` 使用 `dotenv` 来加载你的额外环境变量，例如 `.env` 文件。默认情况下， `.env` 文件从 [`root`](/zh/docs/config/shared#root) 加载，你可以使用 [`envDir`](#envdir) 来自定义。

```js
// .env
FARM_APP_SECRET=secret
Farm_APP_PASSWORD=password
APP_VERSION=1.0.0
```

`Farm` 通过dotenv加载 `.env` 文件，将其加载到 `process.env` 中，并最终将其注入到define中。

:::danger
为了确保客户端的安全，防止当前系统中的环境变量被暴露给客户端， `Farm` 只会识别以 `FARM_`、`VITE_` 开头的重要环境变量，以便更好地兼容vite及其生态系统。前缀可以通过 [`envPrefix`](#envprefix) 前缀配置
:::

`Farm` 通过dotenv-expand扩展环境变量。对于仅用于开发的环境变量，使用 `.env.development` 文件，对于仅用于生产的环境变量，使用 `.env.production` 文件，对于通过 `--mode <stage>` 传递的自定义模式，从 `.env.<stage>` 文件加载。

* 如果你想自定义加载 `.env` 文件的目录，你可以配置 [`envDir`](#envdir)。
* 如果你想自定义注入到 [`define`](/zh/docs/config/compilation-options#define) 的环境变量的前缀，你可以配置 [`envPrefix`](#envprefix)。


## envPrefix

- **默认值**: `FARM_`、`VITE_`

通过配置 `envPrefix` 来自定义环境变量的前缀。以 `envPrefix` 开头的环境变量将自动注入到define中。例如，在 `.env` 文件中：

```js
// .env
FARM_CUSTOM_VERSION=1.0.0
APP_VERSION=0.1.0
```

那么 `FARM_CUSTOM_VERSION` 将被注入到你的业务代码中，但 `APP_VERSION` 不会被注入。在你的业务代码中：

```tsx
export function MyComp() {
  const farmCustomVersion = FARM_CUSTOM_VERSION;
  return <div>Farm Custom Version: {farmCustomVersion}</div>
}
```
`FARM_CUSTOM_VERSION` 将自动被替换为 `'1.0.0'` 。



## envDir
- **默认值**: `<root>`

加载env文件的目录。默认情况下，Farm从根目录加载 [`env 文件`](#env-文件)。

```ts
export defineConfig({
  envDir: './env'
})
```

对于上述配置示例，Farm将从 `<root>/env` 目录加载`.env`、`.env.development`等环境变量文件。
