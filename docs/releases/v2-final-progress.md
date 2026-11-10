# V2 最终流程进度

目标：PR 完整测试；main 按实际源码构建并发布；仅构建增量和可信产物复用，不继承历史测试成功。用户自行合并，不实现通知、自动重跑或跨 registry 原子事务。

状态：待实现 / 已实现待验证 / 已实测 / 阻塞。“已实测”以下均注明本地范围，不代表实际公开发布成功。

实现基线：`c3c99547eaba6624c5fce482db92fbce5995ef37`。固定模板 A：`9f93b5e5359bbf0c3d7f750f7a24469eeaab9b6d`；调用方及实现 B：`77b1cbca4bd1d7c7399a933554386b0619535388`。本轮真实构建/打包均对应 B；后续本文件的进度更新不改变产品、工作流或模板。

本地证据根目录：`/Users/brightwu/Documents/Codex/2026-11-01/task-3/v2-final-flow-20261109/`。

| 节点 | 状态 | 实现、实际证据与剩余边界 |
|---|---|---|
| 1. 现场保护 | 已实测 | 原 14 文件暂停补丁 `before.patch`，SHA256 `f54135e9e8329b9ce5eff6494d8474628e8fa4c82e5771807317bc9805a77f60`；六组旧证据保留。构建生成的 create-farm loader 前后字节及 diff 独立保全，按正式独立打包 checkout 边界恢复提交版；无用户源码丢失 |
| 2. PR 全量测试 | 已实测 | 本地真实 CI 清单 38/38 命令通过（`full-contract-pass3/results.json`）；移除选择/历史等待/version-only 测试跳过，保留完整清单、实际上传和 case-count。普通 Changesets 规则不变。PR 当前 checkout 候选模式覆盖 pending Changesets、源码修改和合法 merge；Linux packaged SDK 与三平台 npm 测试接线保留。新 SHA 的完整托管测试待执行 |
| 3. 构建输入图与缓存键 | 已实测 | 沿用一张依赖图、product_inputs/preparationDeps/measured_context。21 项 catalog 合同及真实 30 个 native 字节的 5 项解析合同通过；包括跨 SHA 同内容命中、单产品及共享锁/toolchain 失效。实际配方重建和 compiler incremental rerun 各 30 件、摘要完全相同；使用已有 Cargo target，不是空缓存构建 |
| 4. main 成功构建上传认证 | 已实现待验证 | 固定 ABI 输出、ID/digest/original attempt、窄 native signer scope 已接线；不依赖整轮发布成功。11 份模板经 git show A 字节比对，13 项签名合同通过；本地模拟 API/crypto，真实 main OIDC 签署未执行 |
| 5. PR cache hit/miss | 已实现待验证 | 多份 partial catalog 累积、缺失补建、无测试继承已通过合同。`real-pack-2/native-real-byte-cache.json` 使用真实产物并执行生产 verified_run/claims/binding/catalog；外部服务和密码学仍模拟，不称真实托管 cache hit。跨平台真实消费待新 CI/main 缓存产生 |
| 6. main 增量发布计划 | 已实测 | 保留版本/registry/既有依赖图和 selection，无新版不额外预热。`registry-current.json` 于 2026-11-09 10:09–10:10 UTC 只读观察：297 npm 精确版本缺失，28 Cargo 中 25 缺失、3 已存在。版本及历史基线未改；发布时仍须重新预检 |
| 7. main 构建打包冻结 | 已实测 | main 无 CI/Unit 等待及消费者功能测试；构建、归档完整性和冻结保留。正式顺序配方两轮各 30 Mac native 成功（`real-formal-native/receipt.json`）；真实 JS 构建与 81 npm 归档通过；25 Cargo 归档默认编译校验通过，根锁不变。本机子集不代表全 297 跨平台产物 |
| 8. 发布事务与读回 | 已实测 | 真实 81 npm 归档进入生产状态机、GitRefStore 和 registry 字节/tag 读回算法：首次、noop、冲突、篡改、响应丢失、最终记录失败恢复通过。`real-pack-2/npm-real-byte-recovery.json`；Git/registry/授权传输仅内存模拟，外部写入零 |
| 9. 原字节恢复 | 已实测 | npm 部分成功与最终记录失败保持原 manifest/归档；25 个真实 Cargo 归档首次/noop/丢响应恢复/冲突/篡改通过（`real-pack-2/cargo-current-frozen-simulation/result.json`）。旧 manifest 消费者语义与新 byte 模式双向隔离；无自动 POST。实际部分公开发布恢复未执行 |
| 10. 清理 | 已实测 | 删除 main consumer workflows、main-evidence recovery、suite selection/baseline 等死入口；删除无 main 生产者的 CI/Unit signer 工作流/模板与 pending scope；保留严格 legacy 读取和有效回归。A+B 共 101 文件，新增 1875 行、删除 4637 行；38 条真实 CI 合同均执行，无旧 workflow 死引用 |
| 11. 本地合同与 dry run | 已实测 | 宽 Node 合同 248 通过/1 Windows-only 跳过；Biome 494 文件通过；源码 CSpell 2491 文件零问题（排除两组保留旧生成证据）；actionlint 无实际错误、5 条已核实工具兼容诊断。真实隔离安装、Core ESM/CJS/native 路径与哈希、create-farm、Docs 内容，以及 Vue/Vue JSX/SDK 编译通过。首轮嵌套 pnpm 错路径失败保留在 `real-pack/`，固定本地 PATH 后 `real-pack-2/` 全绿。未执行本机全仓 ready、全平台业务测试或公开发布 |
| 12. 审查/提交/远端 | 已实现待验证 | A/B 已本地提交并独立只读审查；固定模板/caller/trust 同步。等待最终普通推送及 exact HEAD 远端 CI，不能用旧 c3 绿色代替。用户暂停的普通 PR Changesets 失败、review/approval 等合并条件仍按实际状态保留；不合并、不发布 |

完整证据摘要：`REAL-DRY-RUN-CHECKPOINT.md`。所有本轮自有重型会话均自然结束且 owner 释放共享锁。先前全局进程列表被环境拒绝，未重新查询或声称全局无其他进程。托管身份、真实签署、跨平台完整矩阵及 registry 写入只能在相应真实流程验证，本地 dry run 不保证线上不会失败。

## macOS UnoCSS 测试收尾（2026-11-09）

- 节点 2 / 11：已实测。旧 HEAD `602e89060305cb92c4a5ed4e9c4747a5996f3bd6` 的 macOS 用例超时后，清理会与仍在运行的写入竞争。现在等待 fixture Promise 结束才清理/恢复环境；UnoCSS 依赖初始化放入 setup，只加载同一安装中的必要 preset，默认用例超时不变，50% → 100% → 插件重启后 100% 三项语义保留。
- 本地原用例在 Node 20.20.2 和 22.20.0 均通过，未复现 hosted 的 5 秒耗时，不能认定唯一耗时根因。清理竞争用确定性延迟写入验证：恢复旧顺序时 2 项失败，修复后成功/失败写入两项均通过。
- Node 20.20.2（与失败 hosted 同版）和 Node 22.20.0 的完整 TypeScript 测试分别 60 文件 / 303 项通过；Core、PostCSS、Vue 示例实际构建及 PostCSS 类型检查通过。仅测试/文档及 PostCSS 当前候选摘要变化，无版本、Changesets、发布配置或 native 源码变更。
- 节点 12：已实现待验证。修复后 exact HEAD 的 hosted macOS/全链仍待执行；旧轮成功的 9 native、三平台 Cargo/Examples 不是新 SHA 成绩。真实 OIDC/main-to-PR cache hit/公开 registry 发布仍未验。
- 证据：本地 `unocss-macos-20261109/` 下的 baseline-node20.log、baseline.log、cleanup-red.log、verification.json 与两个 typescript-node*.log。普通 PR Changesets 失败继续不处理；不合并、不发布。

## main 发布验签修复（2026-11-10）

- 节点 2 / 12：#2335 已由用户合并到 `b03f831d9f0a336ef6c0121adcf4891f7bf8e959`。合并前 `f0a77754` 的三平台测试、9 native、完整候选封存/三平台消费者和最终 gate 均通过；普通 PR Changesets 未处理。
- 节点 4 / 8 / 9：发现真实发布阻塞。固定 gh 2.103.3 不允许同时使用 `--cert-identity` 和 `--signer-workflow`。仅删除后者，保留精确 signer URI/revision、digest/source/ref/issuer、托管 runner 和全部签名后策略。
- 已实测：Mac 官方 checksum 固定 CLI 验证原 Cargo proof（run 42730979843 / attempt 1 / artifact 13724513300）成功；错误 source、signer URI、signer digest、篡改 manifest 均拒绝。真实 CLI 回归显式 opt-in，默认合同不联网；证据位于本地 `attestation-cli-20261110/`。
- 已实现待远端验证：新 signer A `7bb6de8e975fdf930925397fa5e46c5a3f78ef50`，caller/trust 同步指向 A，模板内容不变且从 A 逐字节核对。Cargo 后续 attempt 仍只接受同 source/run、新 A 的原冻结证明，不新增旧 signer 信任。恢复调用链合同的 API/crypto 为模拟；旧 9f proof 的实测不代表新 A 已实际签署。
- 阻塞边界：旧 b03 运行重跑仍固定旧代码，不能修好它；本修复不触发重跑或发布。新 A 的 hosted 签署、实际后续 attempt 恢复、npm/Cargo registry 写入仍待合并后真实流程验证。版本、Changesets、权限、冻结原字节和发布锁均未改变。
