# DevServer 和 HMR
Farm默认在 `development` 环境中提供 `DevServer` 并启用了 `HMR` 。

## 配置 Dev Server
Farm提供了许多有用的选项来配置开发服务器。所有的DevServer选项都是通过[`server`](/zh/docs/config/dev-server)配置的。

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  server: {
    port: 9801,
    cors: true,
    proxy: {
      // ...
    },
    open: true,
  }
})
```

:::note
如果你正在为Farm开发工具，请参考[Javascript API](/zh/docs/api/javascript-api)然后以编程方式创建开发服务器。
:::

## Dev Server 中间件
你可以使用 [`middlewares`](/zh/docs/config/dev-server#middlewares) 来处理开发服务器的请求。Farm v2 使用 [connect](https://github.com/senchalabs/connect) 风格的中间件。中间件是一个接受 `Server` 实例并返回 `connect` 处理器 `(req, res, next)` 的函数：

```ts title="farm.config.ts"
import type { IncomingMessage, ServerResponse } from 'node:http';
import { Server, defineConfig } from '@farmfe/core';

export function headers(devServerContext: Server) {
  const { config } = devServerContext;
  if (!config.headers) return;

  return (req: IncomingMessage, res: ServerResponse, next: () => void) => {
    if (config.headers) {
      for (const name in config.headers) {
        res.setHeader(name, config.headers[name] as string | string[]);
      }
    }
    next();
  };
}

export default defineConfig({
  server: {
    middlewares: [
      headers
    ]
  }
})
```

在上述示例中，Farm中间件是一个接收 `Server` 上下文并返回标准 `connect` 风格处理器的函数。常见的 `connect`/`express` 兼容中间件可以直接使用，例如：

```ts {2,7}
import { defineConfig } from "@farmfe/core";
import compression from 'compression';

export default defineConfig({
  server: {
    middlewares: [
      () => compression()
    ]
  },
});
```

## Hot Module Replacement(HMR)
Farm提供了一个与 [兼容 Vite 的HMR API](/zh/docs/api/hmr-api)。如果你是框架作者，可以利用这个 API 来更新你的应用实例，而无需重新加载页面。

HMR API允许你在应用运行时接收模块的更新，并应用这些更新，而无需重新加载整个页面。这可以极大地提高开发效率，因为它允许你在不丢失应用状态的情况下看到代码更改的效果。

* 对于React，官方插件 [@farmfe/plugin-react](/docs/plugins/official-plugins/react)会自动启用 HMR。
* 对于Vue、Solid等框架，它们的插件如 `@vitejs/plugin-vue` 、 `vite-plugin-solid` 等都支持HMR。

Farm提供了官方模板，这些模板已经设置好了所有这些能力，你可以通过create-farm创建应用，然后所有的HMR能力就可用了。

:::note
* 对于应用用户，HMR通常是开箱即用的，如果你需要自定义HMR行为，可以参考 **[兼容 Vite 的 HMR API](/zh/docs/api/hmr-api)**。
* 如果你是框架作者，可以参考 [HMR选项](/zh/docs/config/dev-server#hmr) 来配置HMR。
:::
