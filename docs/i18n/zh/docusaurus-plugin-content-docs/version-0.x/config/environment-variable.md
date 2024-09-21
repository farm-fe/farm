## Environment variable 环境变量

`Farm` 通过 `process.env.NODE_ENV` 来区分开发和生产环境。

在不同环境中, 环境变量会被静态替换, 所以请使用静态的常量来表示环境变量, 而不是动态的表达式.

### `.env` 文件

`Farm` 使用 `dotenv` 来加载您的额外的环境变量, 例如 `.env` 文件.

```js
// .env
FARM_APP_SECRET=secret
Farm_APP_PASSWORD=password
APP_VERSION=1.0.0
```

`Farm` 会通过 dotenv 加载 `.env` 文件, 并且将其加载到 `process.env` 中 最终在 define 中注入.

:::warning
为了保证客户端安全, 防止将当前系统中的环境变量暴露给客户端 `Farm` 只会识别以 `FARM_` 开头和一些重要的环境变量.
:::

`Farm` 通过 dotenv-expand 来拓展环境变量


如果你想自定义 env 变量的前缀，可以配置 `envPrefix`。

### envPrefix env 变量前缀

* **默认值**: `FARM_`

通过配置 `envPrefix` 来自定义 `env` 变量的前缀。

```ts
