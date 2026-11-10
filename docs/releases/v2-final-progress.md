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

## 同仓签名工作流修复（2026-11-10）

本批基线 `7ed329aee81435217047e8554bd500dfc608d1ce`。用户批准 signer 信任从独立固定版本改为本次获准进入 main 的原运行提交；不改变 main 保护、权限、版本或发布范围。

| 本批节点 | 状态 | 证据与边界 |
|---|---|---|
| 现场保护 | 已实测 | 原 Mac 工作区 tracked 干净，六组旧证据保留，新分支从确认 main 创建；本地 `relative-signer-20261110/before-*` |
| 同仓调用与信任 | 本地已实测 | 相对 reusable workflow；checkout 固定 job.workflow_sha。signer/source/caller SHA 必须等于原 API run.head_sha。证书 URI 精确 main ref 加独立 signer digest，不查询最新 main，不信任 manifest 自报身份 |
| 恢复与反例 | 局部实测 | 原 source M 的 attempt 2 仍 signer M，新 aggregationAttempt=2，冻结行保留 attempt 1 artifact/digest/job/origin；main 前进 N 不影响。错误 source/signer/run/attempt/artifact 拒绝，API/crypto 为显式 fixture |
| 本地 dry run | 已实测 | 9 组 Python 合同全部通过，22 项 Node 工作流合同通过；真实 CLI 11/11（含原 proof 和错误身份/篡改拒绝）。旧 proof 在新 same-source 策略下拒绝；不是新相对签名正例。日志位于本地 relative-signer-20261110 |
| 真实 Actions 解析 | 已实测 | [run 42741216158](https://github.com/farm-fe/farm/actions/runs/42741216158)，源码 d386a43b9d60b7c8d76fb20cdbf6dd59161d299d，2026-11-10 02:21:23 UTC 终态 skipped；13 job 全跳过、零步骤，5 个 reusable 引用均为同 SHA，无 Invalid workflow。feature dispatch publish=false、producer_run 为空，无签署/构建/发布；等待旧运行自然结束，未取消 |
| 新 PR 与远端 CI | 已开 PR，CI 未完成 | [Draft #2346](https://github.com/farm-fe/farm/pull/2346)，分支 codex/fix-release-signer-relative-20261110。d386a43 的 hosted source contracts、三平台安全、Type Check、Changesets、Spell、标题通过，Unit/native 等仍运行。本次结果补记仅改文档；不把旧 SHA 成绩冒充后续 SHA。真实相对 OIDC/签名与发布/恢复仍待合并后验证 |

官方语义：相对 workflow 来自 caller 同一提交；`job.workflow_ref` 与 `job.workflow_sha` 分别约束 URI 和实际版本。参考 https://docs.github.com/en/actions/how-tos/reuse-automations/reuse-workflows 和 https://docs.github.com/en/actions/reference/workflows-and-actions/contexts 。旧 b03/7ed 运行保持原代码，不自动迁移；旧 proof 并未损坏，只是不属于新的 same-source release 信任模式。

- 同批新增恢复阻塞：旧 b03 npm run 42730980259 的 publisher 在 registry 写入前因合法 `/compare/<SHA>...<SHA>` 被路径穿越守卫误拒而失败。已补真实 GitHubReadOnly + loopback HTTP 红例，只允许两个精确 40 位 SHA 的 compare 路由，其他 traversal/外域/编码变体拒绝。main 前进后的 release preflight 与原运行恢复改用真实 HTTP JSON 适配器回归；签名归档仍为显式 fixture，不宣称真实旧运行已恢复。最后一轮 110 项受影响 Python 合同和 22 项 Node 合同通过；未改动的 CLI 11 项真实 bundle/4 项固定下载合同已在本批前轮通过。旧运行不重跑。


## npm prerelease latest 修复（2026-11-10）

基线 main `bf4e7e7fa213ceb716634f199e945aea2cfcfe84`，原 npm run `42766527296` / attempt 1 在首轮全量预检失败；Cargo run `42766527334` 已成功。原日志未点名包，按冻结顺序和当前 registry 重放，首个可复现项是第 11 项 replace-dirname-darwin-arm64。

| 本批节点 | 状态 | 证据与边界 |
|---|---|---|
| 现场与生产只读核查 | 已实测 | 原 Mac tracked 干净，六组旧证据保留。原锁 active owner=42766527296/1、phase=uploading、packages 为空，事务 2f86b7aa40e4683b30405d79c2c533f2140cbc78b3ba702b60f4b3ff80ad56d6；未修改锁或产物 |
| SemVer 修复 | 局部实测 | lookup/confirm 共用完整 SemVer 比较，目标仍要求稳定精确版本。使用已验证 Node 捆绑 npm 的 SemVer，不要求 publisher 安装工作区依赖；临时发布 tag 保持不变 |
| 回归与 dry run | 本地已实测 | 原 nightly/RC 在两个入口的红例已保留；最终 130 项 Python（含 45 项 publisher、4 项 HTTP dry run）、165 项 Node 合同全过。297 包真实只读快照预检重放全过。HTTP 读取/字节验证、生产状态机与比较器为真实代码；Git/registry 写入仅内存模拟，归档为明确 fixture |
| 生产恢复方案 | 待单独批准 | 修复合并后、新 main publisher 尚未启动时，重新核 owner 终态/未到上传循环、297 目标仍不存在、锁 revision/fence/transaction 未变；仅以现有 GitRefStore.swap 的一次非强推 CAS 将 active 置空，并按既有机制生成新 operationId；baseline/completed 和旧 Git 历史/冻结 artifacts 保留。不调用不符原因的 reject_unwritten，不自动接管/重跑；当前未执行 |
| 完整合同/审查 | 本地已实测 | 独立只读审查无阻塞；14 条最终命令全过，Biome 493 文件、改动文件 CSpell/diff 检查通过。初轮旧 Python/非标准 Node、loopback 沙箱、失效 pnpm 环境错误均保留；以固定 Node 22.20.0、Python 3.13、pnpm 9.4.0 复验。本地证据目录 prerelease-latest-20261110/final |
| 新 PR/远端 | 已开 PR，远端验证中 | [Draft #2347](https://github.com/farm-fe/farm/pull/2347)，分支 codex/fix-release-prerelease-latest-20261110。源码修复提交 2b283aeb6f10354c419ff9ca861a20f7990426b1 已普通推送并独立 API 核对，本文补记仅更新状态。未合并、未发布、不改版本/Changesets/权限；最终 HEAD hosted CI 与真实发布未验 |

竞态限制：写 tag 前重读、已观察到更高版本则不写、写后验证均保留。npm dist-tag 的读后写不具备条件 CAS，不能保证任意外部维护者并发下绝不瞬时降 tag；本批不改变发布协议或引入新权限。生产快照 297 exact-version 均为 404（41 prerelease latest / 208 stable / 48 无 latest）仅代表采样时刻，未来操作前必须重查；1.27 GB 原 candidates 未重新下载复验，artifact 元数据不等于内部字节/签名复验。


## Registry 预算生命周期修复（2026-11-10）

基线 main `4e722af51789be073bb377a9c1d57f185bfdc26a`。npm run `42785304608` / attempt 1 的 9 个 native、候选和冻结签名成功，publisher 在首轮全量预检超过 reader 从构造累计的 600 秒预算，未进入 npm publish。旧单次解锁授权不覆盖本次新锁；当前只修代码，不执行生产恢复。

| 本批节点 | 状态 | 实际覆盖与边界 |
|---|---|---|
| 现场/证据保护 | 已实测 | 原 Mac tracked 干净开始，六组旧证据保留；独立分支 codex/fix-registry-budget-20261110。生产新事务 fe6d5f2d40339f811c26d6406fb6a37d95a403c4d2c6706986ab7d7ba8a91267 保留 |
| 操作及批次预算 | 本地已实测 | reader 空闲不耗操作预算；metadata 30 秒、单包核验/流式归档 600 秒、完整读回（含 latest）1500 秒、publisher 阶段4500秒。嵌套不能续期；90/30分钟 Actions 上限不改。3GiB累计及100MiB单归档不重置/放宽 |
| 状态读取开销 | 局部实测 | 仅缓存完整验证后的精确commit SHA内容，防变异拷贝；每次仍新读ref并校验owner/fence，拒绝新commit沿用旧缓存。298次读取由1192个GET降至301个；不缓存registry/404，不并发上传/状态写入 |
| 红绿与297包模型 | 本地已实测 | 原reader红例已复现；297包真实生产状态机、比较器、预检/读回模型，外部传输/写入为明确fixture；非网络等待、慢请求/流、整批截止、字节累计和恢复不重发均覆盖。不是生产发布验收 |
| 完整合同/独立审查 | 本地已实测 | 144项Python、165项Node通过；Biome/CSpell/diff通过。独立审查无阻塞，超时字节计数已修；既有ZIP夹具时间引起的摘要失败已保留证据并固定夹具时间。证据目录 registry-budget-20261110/final |
| 新PR/生产恢复 | 已开Draft，恢复待单独批准 | [Draft #2348](https://github.com/farm-fe/farm/pull/2348)，代码提交1c30d43b67cd977f5d608d346157bf4e4a7531f4已普通推送并独立API核对。Hosted CI及真实发布待验。本批不重跑、不解锁、不修改registry；旧run仍是旧代码，新run所需处理新锁另行批准 |

测量证据：deadline-budget-evidence-20261110T104513Z.json（SHA256 bc52172c3934f2b7b7bf9fe643c2dbfe15ccb90ff340207ca9ac63fb656572c2）。Mac首轮594请求193.189秒包含16个超时，唯一一次低并发重读16项均404；并发耗时不等于Hosted串行耗时。冻结归档合计639350827bytes，两遍1278701654bytes，当前不需要放宽3GiB。现有不可变读取缓存不保证GitHub配额或所有网络条件下必然发版；预算是协作式检查而非抢占式取消，真实新main发布仍待验证。


### #2348 后续精简：重复准备工作（2026-11-10）

此项是在预算修复之后按用户新要求实施，不能用上一批结果替代。

| 节点 | 状态 | 证据/边界 |
|---|---|---|
| 纯比较去重 | 本地模型已实测 | 固定Node/代码下只缓存精确(latestVersion,targetVersion)的成功SemVer结果，最多4096项；registry identity/latest仍每次新读，非法结果不缓存 |
| npm准备与清理 | 本地定向已实测 | 本次writer仅解析一次受控CLI、准备一个私有临时目录与0600占位符npmrc；token仅在子进程环境，正常/异常退出清理。写入及安全检查仍串行 |
| 297包前后实测 | 本地已实测 | 同一模型首次发布SemVer启动1485→2、CLI解析594→1、临时目录594→1；registry请求2673、ownership读取1787（含超时负例）及297publish/297tag均不变。Mac真实Node22比较器/CLI启动，墙钟46.40→2.06秒；传输和写入是内存fixture，不是Hosted网络性能 |
| 恢复/完整回归/审查 | 本地已实测 | 最终148项Python、165项Node通过；同reader第二遍不重发、累计字节保留。错key/非法输入、404变新版本、changed bytes、tag变化及配置异常清理均覆盖；独立审查无阻塞，Biome/CSpell/diff通过 |
| PR/真实发布 | 同一Draft #2348 | 本批本地验证完成，随本提交推送同一PR；不另开PR、不处理新生产锁、不重跑或发布。新SHA远端验证不能继承旧SHA成绩 |

证据目录 registry-simplification-20261110，前后benchmark.json保留真实命令启动次数与时间。该精简降低本机重复准备开销，不减少registry请求、锁ref刷新或不可变字节核验，不保证外部网络和API配额永不失败。
