# DevServer 和 HMR
Farm默认在 `development` 环境中提供 `DevServer` 并启用了 `HMR` 。

## 配置 Dev Server
Farm提供了许多有用的选项来配置开发服务器。所有的DevServer选项都是通过[`server`](/zh/docs/config/dev-server)配置的。

```ts
import { defineConfig } from 'farm';

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
你可以使用 [`middlewares`](/zh/docs/config/dev-server#middlewares) 来处理开发服务器的请求。例如：

```ts title="farm.config.ts"
import { Middleware } from 'koa';
import { Server, defineConfig } from 'farm';

export function headers(devSeverContext: Server): Middleware {
  const { config } = devSeverContext;
  if (!config.headers) return;

  return async (ctx, next) => {
    if (config.headers) {
      for (const name in config.headers) {
        ctx.set(name, config.headers[name] as string | string[]);
      }
    }
    await next();
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

在上述示例中，Farm中间件是一个暴露 `Koa Middleware` 的函数。常见的Koa中间件可以直接使用，例如：

```ts {2,7}
import { defineConfig } from "farm";
import compression from 'koa-compress';

export default defineConfig({
  server: {
    middlewares: [
      compression
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
