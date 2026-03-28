---
sidebar_position: 1
---

# 从 Farm v1 迁移到 v2

Farm v2 对配置系统、插件 API 和整体架构进行了重大改进。本指南将帮助你将 Farm v1 项目迁移到 v2。

## 变更概览

Farm v2 包括：
- **新输出格式**：新增 `umd`、`iife`、`system`、`amd` 格式（以及格式数组）
- **增强的插件系统**：新的 Rust 和 JavaScript 插件 hooks
- **服务器中间件重写**：用 connect 风格的中间件替代了 Koa
- **更好的 HMR 处理**：更严格的 origin 校验，新增子选项（`clientPort`、`path`、`timeout`、`overlay`、`protocol`）
- **新功能**：CSS `transformToScript`、CSS modules `localsConversion`、`minify.include`/`exclude`、`persistentCache.envs`、`persistentCache.globalBuiltinCacheKeyStrategy`
- **性能提升**：更快的构建和更好的模块打包

## 破坏性变更

### 配置变更

#### `server` 替代 `devServer`

在 v2 中，开发服务器配置键从 `devServer` 重命名为 `server`：

**v1：**
```ts
// farm.config.ts (v1)
export default {
  devServer: {
    port: 9000,
    hmr: true,
  }
}
```

**v2：**
```ts
// farm.config.ts (v2)
export default {
  server: {
    port: 9000,
    hmr: true,
  }
}
```

#### `server.spa` 被 `server.appType` 替代

布尔选项 `spa` 已被 `appType` 替代：

```ts
// v1: spa: true/false
// v2: appType: 'spa' | 'mpa' | 'custom'
export default {
  server: {
    appType: 'spa', // 默认值
  }
}
```

#### v2 新增 `server` 选项

- `origin` — 设置开发服务器 URL 的 origin
- `allowedHosts` — 限制可访问服务器的主机
- `middlewareMode` — 以中间件模式运行服务器（不创建 HTTP 服务器）
- `preview` — 专用的预览服务器配置

### 服务器中间件 API 变更

**破坏性变更**：Farm v2 使用 [connect](https://github.com/senchalabs/connect) 中间件替代 Koa。

**v1 (Koa)：**
```ts
import { Middleware } from 'koa';

function myMiddleware(server): Middleware {
  return async (ctx, next) => {
    ctx.set('X-Custom', 'value');
    await next();
  };
}
```

**v2 (connect)：**
```ts
function myMiddleware(server) {
  return (req, res, next) => {
    res.setHeader('X-Custom', 'value');
    next();
  };
}
```

请将所有 `koa-*` 中间件包替换为对应的 `connect`/`express` 兼容版本（例如 `koa-compress` → `compression`）。

### HMR Origin 校验

**变更**：HMR 服务器现在会校验客户端的 `Origin` 头。

**v1 行为**：HMR 接受来自任何 origin 的连接

**v2 行为**：HMR 拒绝来自未识别 Origin 头的连接

**如果你遇到 HMR 错误：**
1. 确保客户端和服务器使用匹配的主机名/域名
2. 如有需要，更新你的 HMR 配置：

```ts
// farm.config.ts
export default {
  server: {
    hmr: {
      host: 'localhost',
      port: 9801,
    }
  }
}
```

### HMR 新增子选项

v2 新增了以下 HMR 配置选项：
- `clientPort` — HMR 客户端端口（在代理后面使用时很有用）
- `path` — 自定义 HMR 端点路径（默认：`/__hmr`）
- `timeout` — 连接超时（毫秒）
- `overlay` — 在浏览器中显示/隐藏错误覆盖层（默认：`true`）
- `protocol` — WebSocket 协议（`ws` 或 `wss`）

### 插件 API 变更

#### Rust 插件 API

Rust 插件 trait 新增了以下 hooks。详细信息请参阅 [Rust Plugin API 文档](../api/rust-plugin-api)。

v2 新增 hooks：
- `freeze_module` — 模块冻结后调用
- `module_graph_build_end` — 模块图构建完成时调用
- `freeze_module_graph_meta` — 冻结模块图元数据
- `process_resource_pots` — 打包后处理 resource pots
- `process_rendered_resource_pot` — 渲染后处理 resource pot
- `augment_resource_pot_hash` — 增强 resource pot 哈希值
- `process_generated_resources` — 生成后处理资源
- `handle_entry_resource` — 处理入口资源
- `module_graph_updated` — HMR 期间模块图更新后调用
- `update_finished` — HMR 更新完成时调用
- `handle_persistent_cached_module` — 从持久缓存加载模块时调用

#### JavaScript 插件 API

JavaScript 插件接口新增了以下 hooks。详细信息请参阅 [JavaScript Plugin API 文档](../api/js-plugin-api)。

v2 新增 hooks：
- `freezeModule` — 模块冻结后调用
- `processRenderedResourcePot` — 渲染后处理 resource pot
- `augmentResourcePotHash` — 增强 resource pot 哈希值
- `finalizeResources` — 在输出前最终化所有资源
- `updateFinished` — HMR 更新完成时调用

## 迁移步骤

### 1. 更新 Farm 版本

```bash
# 更新到 Farm v2
npm install @farmfe/core@latest @farmfe/cli@latest
# 或
pnpm install @farmfe/core@latest @farmfe/cli@latest
```

### 2. 更新配置文件

检查你的 `farm.config.ts` 或 `farm.config.js`：

1. **将 `devServer` 重命名为 `server`**
   ```ts
   // 之前 (v1)
   export default { devServer: { port: 9000 } }
   // 之后 (v2)
   export default { server: { port: 9000 } }
   ```

2. **将 `spa` 替换为 `appType`**
   ```ts
   // 之前 (v1)
   server: { spa: true }
   // 之后 (v2)
   server: { appType: 'spa' }
   ```

3. **将中间件更新为 connect 风格**（参见 [服务器中间件 API 变更](#服务器中间件-api-变更)）

4. **测试配置有效性**
   ```bash
   farm build
   ```

### 3. 更新插件

如果你使用了自定义插件：

1. **Rust 插件**：查看 [Rust Plugin API 文档](../api/rust-plugin-api) 了解 hook 签名变更
2. **JavaScript 插件**：查看 [JavaScript Plugin API 文档](../api/js-plugin-api) 了解 hook 签名变更

对于官方插件，确保它们兼容 v2：

```bash
npm install @farmfe/plugin-{feature}@latest
```
