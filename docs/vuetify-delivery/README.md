# Vuetify 4.2.5 源码修复交付包

此目录交付完整源码补丁及严格消费回归用例。目标为官方提交 `44521c37f5ee2587dc7a6635d099ff40488bc32e`；它不是 npm fork、已合并上游修复或 Farm 正式依赖。实验包运行时仍为官方相同字节，只重新生成声明。

## 包含什么

`vuetify-4.2.5-complete-source.patch` 包含 13 个源码/生成器文件的修复：共享全局组件/指令类型、聚合声明的 shim 处理、directive 的 nullable prevVNode、VSelect/VDateInput 相关类型，以及 DataTable 六文件有限列键与槽名推导。它包含先前 GlobalComponents 补丁和最新 DataTable 补丁，不能与两份增量补丁重复应用。

`manifest.json` 记录每个文件的官方原始 SHA256、修复后 SHA256、生成依赖的固定版本、DOM 标准库摘要及通过严格消费的归档摘要。ZIP 中附带五类组件的公开契约 fixture 和八种额外调用形式 fixture。

## 应用与重新生成

1. 在上述官方提交的新 checkout 中执行 `git -c core.autocrlf=false -c core.eol=lf apply --check <补丁绝对路径>`，再去掉 `--check` 应用。先核对 manifest 中的原始摘要，避免把其他版本的源码误当作基线。
2. 使用 manifest 固定的生成依赖，包括 Vue 3.5.44、TypeScript 6.0.2、native-preview 7.0.0-dev.20260323.1、Rollup 4.55.1 和 rollup-plugin-dts 6.4.1。另一次全新工具环境已通过实际 npm 安装验证：没有工具 junction，直接依赖逐项核对版本与真实目录，599 个生成声明与严格消费通过版逐文件字节一致。
3. 声明生产端另需提供 TypeScript **5.9.3** 的原始 `lib.dom.d.ts`、`lib.dom.iterable.d.ts`、`lib.dom.asynciterable.d.ts`，分别作为 `@typescript/lib-dom` 的 `index.d.ts`、`iterable.d.ts`、`asynciterable.d.ts`，核对 manifest 摘要。这是生成标准库设置，不是手改生成声明。native-preview 默认关闭替换，必须显式开启。
4. 在 `packages/vuetify` 中依次运行固定工具的 `tsgo --emitDeclarationOnly --libReplacement true --lib ESNext,DOM,DOM.Iterable,DOM.AsyncIterable -p tsconfig.dist.json`、`node build/transform-types.js`、`rollup --config build/rollup.types.config.js`。官方 dist/json 输入也必须存在；本轮来自相同官方包，未验证全量运行时构建。
5. 通过实际 npm tarball 安装验证，不能把源码生成成功当作交付完成。消费者使用 TS 5.9.3、Vue 3.5.44、vue-router 4.6.3、strict=true、skipLibCheck=false、NodeNext；检查 standard、Node-only、labs、bundle 及公开槽契约，并移除八项 `@ts-expect-error` 确认准确产生 TS2339×3、TS2322×4、TS2345×1。

## 已知边界与发布路径

独立复现证据为 Farm 的 `docs/release-evidence/v2-vuetify-fresh-generator-resumed-2.json`，驱动为 `scripts/v2-vuetify-fresh-generator.mjs`。在具备记录中官方源码和 dist/json 输入的 Farm 工作区，可设置新的 `FARM_VUETIFY_FRESH_RUN` 名称运行，避免覆盖旧证据。此驱动使用本机 Node 22.20 和 Windows NTFS 压缩；不是跨平台或完全不依赖已有源码输入的下载引导器。

首次与一次恢复安装均因 ENOSPC 失败，原始报告 `v2-vuetify-fresh-generator.json`、`v2-vuetify-fresh-generator-resumed.json` 保留。对已结束验收的包与闲置 Rust 构建依赖完成压缩前后哈希核对、释放空间后，在同一临时目录恢复，实际 npm 安装 79 个包成功。DOM 基线来自全新安装的 `typescript-dom-baseline: npm:typescript@5.9.3`，没有复用原工具目录。此环境不用于性能比较。

八种额外槽调用形式中，六种修复前支持的形式均通过。普通 JSX child 对象和 child 函数在修复前后都存在 TS7006 / TS2578 上下文类型限制；它们是保留的已知边界，不是通过项。

推荐将源码修复与 DOM 生产基线作为可独立审查的上游改动，等待官方版本后再更新 Farm 的依赖与候选包。仓库级 pnpm patch 不能自动修复 npm 下游消费者；本机 file: tarball 或消费者 override 也不能证明正式 npm 交付。此交付包未向外部仓库提交或发布。

完整实测与历史失败保留在 Farm 的 `docs/v2-vuetify-finite-slot-repair.md`。当前 Farm 实际候选安装仍有 Vuetify 根声明失败；后续必须在真实候选安装树重新验证全部插件入口及业务。
