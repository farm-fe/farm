# SSR Toolkit Example

这个 example 用一个完整的 Vue SSR 项目演示 `@farmfe/ssr` 的三种接入方式：

1. 原生 Farm CLI（`farm ssr dev/build/preview`）
2. 用户自定义 Node Host（把 Farm 当 middleware 挂进去）
3. 用户已有模板体系（示例里用 `ejs` 风格模板）

## Tech Stack

1. Vue 3 + Vue Router
2. Vue SFC (`src/pages/*.vue`)
3. Less (`src/styles/app.less`)
4. TypeScript 入口（`src/main.ts` / `src/entry-server.ts`）

当前 example 为保证 preview 懒加载路由稳定，client 构建使用了 `compilation.minify=false`（已记录为 Farm 核心已知问题，后续可在核心修复后恢复）。

## Usage A: Farm CLI

先在仓库根目录构建 CLI：

```bash
scripts/pnpm-node22.sh --filter @farmfe/cli build
```

然后在 `examples/ssr-toolkit` 目录执行：

```bash
pnpm run cli:prepare
pnpm run cli:dev
pnpm run cli:build
pnpm run cli:preview
```

`cli:preview` 已内置 `cli:build` 前置步骤，避免产物缺失导致 preview 失败。

## Usage B: Custom Node Host + Middleware

`examples/ssr-toolkit/server.mjs` 展示了“外部 host 持有自己的 API，Farm 只处理 SSR + assets”的模式。

```bash
pnpm run host:dev
```

说明：  
host 路径本身就是“用户自有 Node server 启动”，所以入口命令是 `node server.mjs`，不是 `farm ssr dev`。  
`farm` 在这个路径里负责构建与中间件能力，不接管宿主服务进程。

如果要指定 HMR 端口：

```bash
SSR_HMR_PORT=9821 pnpm run host:dev
```

如果要指定 host 监听端口：

```bash
SSR_HOST_PORT=3011 pnpm run host:dev
```

关键点：

1. `/api/ping` 由 host 控制
2. `ssrServer.middlewares(req, res, next)` 处理页面与静态资源

为了便于参考，host 逻辑已拆分到 `server/` 目录：

1. `server/runtime-config.mjs`：环境变量与 command/mode/template 解析
2. `server/ports.mjs`：host/HMR 端口策略
3. `server/template.mjs`：dev/preview 模板策略
4. `server/ssr-options.mjs`：SSR options 与启动信息拼装

## Usage C: EJS Template Integration

同一个 host，切到 ejs 模板模式：

```bash
pnpm run host:dev:ejs
```

Host preview 路径会先执行 `farm ssr build` 再启动宿主服务：

```bash
pnpm run host:preview
```

注意：当前 example 的 preview 仅支持 `html` 模板；`SSR_TEMPLATE_MODE=ejs` 仅用于 dev 场景。

当前示例支持两种模板：

1. `index.html`（默认）
2. `index.ejs`（通过 `template.load + template.transform`）

## Route Contract

1. `/` -> `route:home`
2. `/about` -> `route:about`
3. `/products` -> `route:products`
4. unknown path -> `route:not-found` + `#route-not-found`（业务 404 页面）

## HMR Contract

1. Server HMR：修改 `src/entry-server.ts`，SSR HTML 会更新
2. Client HMR：修改 `src/pages/HomePage.vue`，浏览器已打开页面会热更新

## Smoke

```bash
pnpm run smoke
```

覆盖：

1. middleware-dev（含 server/client HMR）
2. middleware-preview
3. cli build + cli preview
4. host API 边界（middleware 有 `/api/ping`，cli-preview 无）
5. less SSR 输出标记（`less-theme-active`）

可选：开启 preview 浏览器级断言（用于排查懒加载路由与样式）：

```bash
SSR_SMOKE_PREVIEW_BROWSER=1 pnpm run smoke
```

## 端口占用处理

常见报错：

1. `WebSocket server error: Port is already in use`
2. `listen EADDRINUSE: address already in use 127.0.0.1:3011`

处理策略（本 example 已实现）：

1. 默认自动探测可用 HMR 端口（从 `9811` 开始向后探测）。
2. 默认自动探测可用 host 端口（从 `3011` 开始向后探测）。
3. 允许用户显式指定：`SSR_HMR_PORT=<port>` / `SSR_HOST_PORT=<port>`。
4. 若显式端口被占用，直接失败并提示，避免静默漂移。

这和 Vite/Nuxt 的经验一致：  
优先“自动选择可用端口 + 保留手动固定能力”，并在端口冲突时给出明确错误信息，方便排查僵尸进程或多实例并行启动。
