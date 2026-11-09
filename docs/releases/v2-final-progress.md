# V2 最终流程进度

目标：PR 完整测试；main 按实际源码构建并发布；仅构建增量和可信产物复用，不继承历史测试成功。用户自行合并，不实现通知、自动重跑或跨 registry 原子事务。

状态：待实现 / 已实现待验证 / 已实测 / 阻塞。当前基线 `c3c99547eaba6624c5fce482db92fbce5995ef37`；未完成提案不得发布。

| 节点 | 状态 | 实现与验收依据 / 剩余事项 |
|---|---|---|
| 1. 现场保护 | 已实测 | 14 文件暂停补丁保存于本地 `v2-final-flow-20261109/before.patch`，SHA256 `f54135e9e8329b9ce5eff6494d8474628e8fa4c82e5771807317bc9805a77f60`；本地/远端基线一致，六组旧证据保留 |
| 2. PR 全量测试 | 已实现待验证 | ci/rust-test/ci-primary-platform：移除 suite selection、历史测试 baseline 等待及 version-only 测试跳过；普通 Changesets 规则不改；105 项流程合同、34 项全量发现/执行合同及 12 项原 attempt 合同局部通过（pr-main-flow-3.log、full-discovery.log、full-suite-attempts.log）；候选 npm、Linux packaged SDK 与三平台 npm 消费者接入 PR；PR 专用当前 checkout inventory 已实现，允许未消费 Changesets/正常源码差异，17 项局部合同通过；最终完整合同与新 SHA 全平台执行待验 |
| 3. 构建输入图与缓存键 | 已实现待验证 | 保留 product_inputs/preparationDeps/measured_context；验证依赖、锁、recipe/features/toolchain/ABI 失效 |
| 4. main 成功构建上传认证 | 已实现待验证 | 固定 ABI 输出、upload ID/digest/original attempt；现有 signer 增窄 native scope，不依赖整轮发布成功 |
| 5. PR cache hit/miss | 已实现待验证 | formal-native-products 累积可信产品命中，缺失补建；两种路径完整测试相同 |
| 6. main 增量发布计划 | 已实现待验证 | 既有 source/registry/依赖图保留；无新版不额外预热；版本与 selection 不变 |
| 7. main 构建打包冻结 | 已实现待验证 | 已去重复 CI/Unit 等待及 main candidate/public 三平台测试；PR 复用已有候选归档测试。release signer A 已本地冻结，B 调用方/pin 已接线，待完整验证；不可推送半套 |
| 8. 发布事务与读回 | 已实现待验证 | 保留锁/fence/逐包冲突/响应不确定读回；单次 registry 字节/tag 读回不声称测试通过。模式只由冻结 manifest 选择，双向混用拒绝；记录 6、入口 16、基线 10、绑定 5、发布 35 项局部通过（*-policy-separated*.log） |
| 9. 原字节恢复 | 已实现待完整验证 | 保留冻结事务及 legacy 消费者语义；27 项原 attempt/恢复合同通过（frozen-attempt-move-2.log）。main-evidence 建议控制器已移除，无自动 POST；真实部分发布恢复仍未执行 |
| 10. 清理 | 已实现待完整验证 | 已删除 main 三平台 consumer workflows、main-evidence recovery 控制器、functional-upload、suite-selection/baseline；保留全量发现、失败传播及 legacy 读取回归；无调用 inspect-* CLI 与伪 cleanup 字段已清理。已删除无 main 生产者的 CI/Unit signer 工作流/模板、main 测试等待 CLI 和 pending 签署 scope；历史证明只读策略保留 |
| 11. 本地合同与 dry run | 已实现待完整验证 | 实际 CI 清单第三轮 38/38 命令通过，包括 git show A 的固定模板检查；宽 Node 合同 248 通过/1 Windows-only 跳过，记录 full-contract-pass3/results.json。源码 CSpell 2491 文件零问题（排除保留的两组旧生成证据），Biome 494 文件通过，actionlint 无实际错误、5 条已明确兼容诊断。上述仅本地合同/模拟；最终完整合同、真实 cold/hot native/pack 与生产算法 dry run 尚未完成；hosted OIDC 与 registry 写入未执行 |
| 12. 审查/提交/远端 | 待实现 | 本地 A `9f93b5e5359bbf0c3d7f750f7a24469eeaab9b6d`；生产 release trust/caller 指向 A，历史 CI trust 保留旧 revision 只读。B 已完成合同验证待本地提交，尚未推送；远端 exact HEAD 待重新验收 |

本地旧 c3 checkpoint 有独立合同/native/browser记录，但不是本方案验收。全局进程列表先前被环境拒绝，不宣称无其他进程；重型任务仍需原子锁和既有协调。
