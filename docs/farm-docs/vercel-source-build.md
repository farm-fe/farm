> RETIRED HISTORICAL DESIGN: The source-build deployment below exceeded Vercel's 45-minute limit and must not be executed. The compatibility command now fails explicitly. Product CI/local source verification remains separate; deployment requires approved published v2 packages and precompiled native payloads. See registry-release-plan.draft.json and scripts/farm-docs-registry-consumer.mjs. No production deployment or merge is authorized until final CI/local preview and the external Vercel side-effect gate are satisfied.

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

## 通用 CI 消费者前置

旧 head 的 examples jobs 在 Linux/Windows 均报 `@farmfe/docs/dist/index.js` 缺失。通用 examples 构建器现在只在选中的示例依赖 `@farmfe/docs` 时，从其实际 manifest 纳入七个原生插件依赖，并在首次 Docs 示例之前依次构建 plugin-tools、缺少的原生产物、Docs 与默认主题公共包。已有 CI 原生产物继续走原有复用判断；没有的插件通过其公开构建脚本生成。非 Docs 示例不引入这些前置。Windows 正常单示例入口已通过，本地复用了原生插件，不能据此宣称远端缺产物的源码构建已通过。

TS CI 缺少 plugin-tools 的构建步骤，导致六个 artifact-copy 用例无法导入 dist；工作流已补公开 build。Windows 相对根目录测试改在 checkout 所在盘创建临时目录，以保证真实相对路径，未改变产品路径逻辑。freezeModule 快照的一行 source map 文件名由旧转义形式改为原始 `index.ts?foo=bar`，与用例原有的显式 sources 断言一致；其余字段及映射不变。候选通过正常单用例更新生成，再在非更新模式下验证；没有批量更新快照。

本地后续验证：example 前置计划 2 项通过，正常 `docs-features` 单示例入口通过（CI 模式复用已有原生文件）；JS/TS 全套单 worker 在 CI 模式且无 `NO_COLOR` 的环境下 39 文件/150 测试通过，没有快照更新或 worker 异常。最初本地 `NO_COLOR` 造成的五个颜色用例失败日志保留。

## 缓存回归的临时目录身份

新 macOS 单元 job 119068225252 在 `update_cache` 的初始 `has_cache(styles.css)` 断言失败；尚未执行显式更新断言。测试配置使用 OS 临时目录的原始路径，而 resolver 默认对目录及祖先调用 `follow_symlinks`，模块 ID 再相对 config.root 计算。测试现在以同一 helper 规范化已创建的根目录，并保留全部缓存命中、更新内容、mtime 和未变 sibling 断言；补充失败时的根目录诊断。产品缓存与解析代码未改。

Windows 正常临时路径的两个测试通过。Windows junction 探针在修复前后均通过，因此没有提供 macOS 差异的红绿复现；真目录 symlink 创建受到本机权限限制，没有提升权限。以上是基于路径身份契约的测试设置修正，macOS 通过仍须后续 CI 实证，不把 Windows 成功替代为 macOS 验收。

另补 Unix 受控回归：在 canonical 临时父目录内创建指向同级 physical 目录的真实相对 symlink，在独立测试子进程中将 TMPDIR 指向该 alias，精确执行原 hash-cache 回归并要求实际运行一项测试。子进程不修改主测试进程的环境，全部原缓存断言仍执行；父目录提前 canonicalize，避免人为叠加宿主 /var 祖先别名；受控相对 link 使用当前 resolver 的相对路径解析契约。该 Unix 用例不在 Windows 本地执行，等待 Linux/macOS CI 验证。
