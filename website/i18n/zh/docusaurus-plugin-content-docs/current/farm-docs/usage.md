# 使用 Farm Docs

**预发布文档。** 本指南描述当前 v2 候选。下列命令用于已具备匹配依赖的项目；对外使用需先有经过验证的正式包。

## 最小项目

```text
my-docs/
  package.json
  farm.config.ts
  docs/
    index.md
```

在 `docs/index.md` 写入标题与内容，然后创建 `farm.config.ts`：

```ts
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig({
  title: 'My documentation',
  url: 'https://example.com',
  basePath: '/',
  docs: {
    collections: [{ id: 'main', dir: 'docs', routeBase: '/' }]
  }
}, {
  compilation: { output: { path: 'build' } }
});
```

第一个参数描述站点，第二个参数描述 Farm 编译选项。`defineDocsConfig` 会注册 Docs 工作流；仅添加原生 Docs 插件不会启用静态生成。

## 内容与配置

每个集合显式声明 `id`、内容目录 `dir` 和 `routeBase`。路径从站点根目录解析。语言和版本分别使用集合，相关集合使用相同 `group`，对应翻译文档使用相同逻辑 ID。允许缺失翻译回退时，显式配置 `fallbackLocale`。

集合的 `sidebar` 指向站点相对路径的侧栏 JSON，控制分类和文档顺序。主题应读取生成的导航目标与 alternates，不要通过字符串替换推导语言或版本地址。

`@farmfe/docs` 随包提供的 Rust 生成类型与 schema 是字段定义依据。应修正不支持的字段和内容错误，不以 JavaScript 编译回退隐藏问题。

## 主题与扩展

通过 `theme.components` 引用站点内 React 模块，通过 `theme.customCss` 添加样式路径。模块可导出 `Layout`、`DocPage`、`BlogPage`、`Navbar`、`Sidebar`、`Toc`、`Footer` 和 `NotFound`，省略的插槽保留默认实现；默认导出替换 `Layout`。`NotFound` 使用独立的 props 类型。

```tsx
import type { ThemeProps } from '@farmfe/docs/theme';

export function Footer({ site }: ThemeProps) {
  return <footer>{site.title}</footer>;
}
```

自定义布局应传递完整 props 并渲染 `props.components` 中的插槽。复用默认阅读样式时保留 `.farm-docs-theme`。仅浏览器操作放在 effect 中，保证服务端输出与浏览器首轮渲染一致。默认主题支持跟随系统、浅色与深色。

公开入口包括提供页面上下文与导航的 `@farmfe/docs/client`、内容组件契约 `@farmfe/docs/mdx`、代码与命令组件 `@farmfe/docs/snippets`、主题插槽类型 `@farmfe/docs/theme`。第二参数中的 `plugins` 接受原生 Rust 插件描述；Docs 配置契约拒绝 Vite 与 JavaScript 编译插件。

## 开发、构建与预览

使用项目实际安装的 CLI 添加脚本：

```json
{
  "scripts": {
    "dev": "farm start",
    "build": "farm build",
    "preview": "farm preview"
  }
}
```

写作时运行开发服务；先生产构建，再运行本地 preview。Preview 提供生成后的静态内容，不等于线上部署。输出目录应是站点根目录内专用目录；工作流会拒绝把无关的非空目录当成旧 Docs 输出。

在真实产物上检查路由、双语、资源、导航、404 与第三方许可声明。报告错误时保留源码位置和原生诊断码。工作区源码预览、打包消费者验证、安装已发布包的部署应分别记录，不能相互替代。

## 发行与安装注意事项

包声明要求 Node.js 20 或更新版本，以及 React/React DOM 18 peer 依赖。`@farmfe/docs`、`@farmfe/docs-theme-default`、`@farmfe/cli`、Core 与原生插件必须来自同一套经过验证的分发。正式 v2 包发布是独立前置条件，本指南不代表已有可立即安装的发行版。

匹配正式包可用后，应启用正常生命周期脚本和 optional 依赖安装，固定验证过的版本并提交锁文件。不要使用工作区链接或任意原生二进制替代外部消费者验收。

## 迁移 Docusaurus 站点

迁移通过站点的 Farm 配置和明确的组件替换完成。Farm Docs 不提供 Docusaurus 兼容包，也不承诺完整兼容。

`@site` 等路径可使用 `compilation.resolve.alias`。站点专用的小适配留在站点中，不加入框架：

```ts
import { fileURLToPath } from 'node:url';
import { defineDocsConfig } from '@farmfe/docs';

export default defineDocsConfig(site, {
  compilation: {
    resolve: { alias: { '@site': fileURLToPath(new URL('.', import.meta.url)) } }
  }
});
```

将 `@theme/Tabs`、`@theme/TabItem` 改为从 `@farmfe/docs/mdx` 命名导入 `Tabs`、`TabItem`，每个项显式提供 `value` 与 `label`。将 `@theme/CodeBlock` 改为从 `@farmfe/docs/snippets` 命名导入 `CodeBlock` 标记；代码必须是静态内容，由已有 Rust snippets 编译器编译和高亮。

导航、站点和页面数据使用 `@farmfe/docs/client` 中有文档契约的 `Link`、`useSite`、`usePage`、`useLocale`。不支持的 Docusaurus context、翻译和主题内部 API 按实际需要改为站点代码。Alias 只负责解析路径，不能复现其他框架的运行时语义或 JavaScript 插件生命周期。替换后验收路由、SSR、主题控件、标签与代码输出；不需要 Docusaurus 运行时或构建依赖。
