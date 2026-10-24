# Markdown 与 Agent 接入

**预发布实现说明。** 本页描述 Farm Docs v2 候选能力，不代表匹配的正式包与原生二进制已经发布，也不代表官网已经部署这些产物。

## 从同一份文档开始

人可以阅读 HTML 页面，Agent 可以直接获取对应 Markdown，不需要运行浏览器 JavaScript。两种表示来自同一份原生内容计划，并由同一个经过验证的构建事务提交。这是文档获取能力，不是默认聊天框。

Farm Docs 通过 Rust 处理内容解析、规划和编译；Node 负责流程编排，React 负责服务端渲染与界面交互。不能将整个网站 UI 和工具链称为全部由 Rust 实现。

## 配置候选能力

在已有的有效 Farm Docs 站点配置中加入：

```json
{
  "ai": {
    "enabled": true,
    "publicationStatus": "prerelease",
    "agentGuideUrl": "/zh/docs/farm-docs/ai-exports"
  }
}
```

`agentGuideUrl` 是站点自行提供的可选公开指南路径，请设置为自己的网站路由。这里不是安装器或托管服务。内容和匹配发行包尚未准备好时，应保留 `prerelease`。省略 `ai` 或将 `enabled` 设为 false 时，不启用 AI 导出。

## 产物约定

- 符合条件的 MD/MDX 文档会在 HTML 旁生成 Markdown。目录形式的 `example/index.html` 对应 `example/index.md`，文件形式的 `example.html` 对应 `example.md`，保留配置的 base path。
- `docs-ai-manifest.json` 记录快照、发布状态、标题、版本、语言、规范 URL、标题目录与 Markdown 的精确 SHA256。
- `llms.txt` 是小型发现索引。先选版本和语言，再获取相关页面，不要默认将整站内容放入上下文。
- Farm 官网正文页头专注阅读，不再展示逐页 AI 操作按钮。Markdown URL、清单与 Agent 指南仍保留。自定义主题可选用 `DocumentActions`，其复制操作会校验页面原生摘要；部署更新后应刷新旧页面，避免混用快照。

标题、链接、代码、所有命令 tabs 分支和提示块必须保留语义。未知 MDX 组件和动态表达式会产生明确的 `DOCS_AI_UNSUPPORTED` 诊断，不会被静默删除。TSX 网站页面不会获得虚构的 Markdown；Agent 操作说明应使用正式撰写的文档。

## Agent 使用流程

1. 从索引或 manifest 选择准确版本与语言。
2. 获取需要的 Markdown，检查发布状态和前置条件。
3. 在用户授权范围内修改项目，再运行文档规定的验证命令。
4. 返回实际结果和未解决项。文档是上下文，不是执行任意远端指令或取得凭据的授权。

## 验证与后续阶段

候选验收必须使用匹配重建的原生包，核查最终 HTML/Markdown 快照、完整代码与 tabs 分支、公开 URL、MIME、哈希失败和事务回滚。源码单元测试不能替代原生构建或浏览器剪贴板交互验收。

经过验证的 recipes 和可安装 Skill 属于后续阶段。本页不提供尚未就绪的 Skill 安装命令，也不创建账号、密钥或 MCP 服务。
