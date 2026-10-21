# Vercel Preview 从源码构建官网

## 触发问题

实际部署在 `/vercel/path0/packages/cli/bin/farm.mjs` 导入 CLI 的 `dist/index.js` 时失败。现有 Vercel 项目的 Root Directory 为 `website`，而 `website/package.json` 的普通 `build` 直接执行 Farm；workspace 安装只建立链接，不会自动生成所有工作区包的 JS 和原生构建产物。

## 仓库内入口

`website/vercel.json` 显式使用 Other framework、workspace 安装脚本、完整构建脚本和 `build-farm` 输出目录；不修改 Vercel 项目或生产设置。`website/package.json` 选择 Node 22 主版本（Vercel 自动选择其维护的小版本），根 packageManager 固定 pnpm 9.4.0。

安装入口从脚本位置确定仓库根目录，执行 `corepack pnpm install --frozen-lockfile`，跳过与静态官网无关的 Electron/Playwright 浏览器下载。正常依赖生命周期仍执行。

构建入口读取仓库 `rust-toolchain.toml`；仅在 Linux 缺少 rustup 时下载官方 `https://sh.rustup.rs` 安装器，再安装仓库固定工具链与组件。依次执行公开 npm scripts：

1. utils、runtime、runtime-plugin-hmr、runtime-plugin-import-meta、plugin-tools。
2. Core 的 `build:rs`。
3. replace-dirname、react、mdx、ssr-css、syntax-highlight、sass、tailwindcss、docs、docs-snapshot、docs-seo、docs-snippets 共 11 个 Rust 插件。
4. Core、CLI、Docs、默认主题的公开 JS 构建。
5. 验证 Core 实际加载、11 个插件的本 checkout 路径/非空文件/SHA256 和 CLI `--help` 可运行，再执行官网 `build:farm` 和实际 NOTICE 输出检查。官网构建加载对应的 Rust 插件。

每一阶段均有开始、成功及失败日志；任一阶段失败立即停止。不会因已有 dist 或 native 文件而跳过正常构建。默认 Cargo 并行度为 2、Node heap 为 2048 MiB，允许环境显式覆盖。编译扩展仍由 Rust 插件实现，Node 脚本仅负责调度、依赖安装和验证。

## 验证与边界

`node --test scripts/farm-docs-vercel-build.test.mjs` 检查真实公开 scripts、关键依赖顺序、前置失败停止和部署配置；`node scripts/farm-docs-vercel-build.mjs --plan` 仅列计划，不执行构建。Windows 已验证过完整原生闭包，不为部署脚本重复所有未变原生构建；Linux 干净部署以本次 Preview 和 Docs CI 的实际日志为准。

Vercel 的编译工具、网络、磁盘、构建时限及访问保护均由实际运行确认。入口正确不等于部署已通过；不得使用本地缓存成功代替远端干净环境验收。未运行版本更新、npm 发布、合并或生产部署。

官方配置依据：https://vercel.com/docs/project-configuration/vercel-json

## 大小写敏感源码检查

后续 Vercel 日志已进入 `build:farm`，报 TeamMembers 无法解析 `../Card`。Git 实际跟踪的是 `components/card/index.tsx`；Benchmark 存在同样引用，两个 import 已统一为 `../card`。文件并未漏提交，也不是缺少 pnpm 包。

部署在准备 Rust 工具链之前运行 `scripts/farm-docs-source-paths.mjs`：以 `git ls-files` 的精确大小写清单检查 `website/src` 的静态 import/export 与字面量动态 import，支持直接文件、扩展名、目录 index 和资源 query。使用 TypeScript 解析源码，忽略注释中的伪 import；不依赖 Windows 的 `existsSync`。此检查仅涵盖相对字面量引用，不验证 alias、包导出或运行时生成路径，实际官网构建仍必需。现有部署测试在两种 CI 系统上执行相同检查，包含错大小写、缺文件、正确 index 与资源路径回归。

Windows 修复后检查点：精确路径检查覆盖 45 条相对引用，部署测试 6/6；单任务重建 Core 与公开 Core 包、验证产物、官网和 NOTICE 均通过。真实预览覆盖 334 路由、16 项既有浏览器观察、NOTICE 原文字节及双语未知路径 404。11 个外部原生插件复用并核对哈希；这不替代新的 Linux CI 或 Vercel 实际部署。
