# 官网第三方 NOTICE 落地与后续验收

## 已完成，不依赖构建

- `website/static/THIRD_PARTY_NOTICES.txt` 包含已映射的 75 个浏览器依赖版本的原始许可文本，以及 Rust Tailwind CSS 生成器模板的原始许可。
- `website/licenses/` 按 SHA256 保存原文字节，`website/third-party-notices.manifest.json` 记录包名、版本、摘要及固定提交的上游原文链接；不包含本机绝对路径。
- 官网自定义 Footer 增加 NOTICE 链接，复用现有 Rust 静态目录复制流程。
- `scripts/farm-docs-notices.test.mjs --source-only` 校验源文件摘要、75 个唯一包版本、每份原文完整嵌入及 CSS 原文；不执行编译。
- CI 在官网构建后运行同一脚本，额外检查输出 NOTICE 字节与首页链接。

## 统一构建验收顺序

1. 资源可用后，仅执行官网公开 `npm run build`；已有 21 个成功的 JS/原生前置无需重跑，除非实际修改相应源代码。
2. 运行 `node scripts/farm-docs-notices.test.mjs`，检查实际输出。
3. 对新快照运行既有官网预览用例，覆盖 334 路由、中英文、当前/1.x、JS/无 JS、移动端、控制台及请求失败；额外检查 NOTICE HTTP 200、正文摘要、页脚链接，以及中英文未知路由 404。
4. 对比原生二进制基线，记录实际新快照、输出文件清单及日志；旧 v08 的构建/浏览器结果只代表旧源码，不代替本次验收。
5. 正常 hooks 补充提交；外部推送/PR 依据会话中已有明确授权执行，不合并或发布生产。

## 范围与边界

531 个旧快照注册模块已完成来源归并，其中 75 个第三方包版本对应本次包级许可清单。此数量不是未来构建自动发现的依赖清单；依赖变化需要更新映射、原文和 manifest。既有图片保持仓库来源与本次未改范围，不由包级清单推导历史图片授权结论。CSS attribution 基于实际 Rust 生成器原文，不把本机可 resolve 的旧 npm Tailwind 版本当成 CSS 输出来源。Linux 暂不本地运行，按用户要求留到 CI。
