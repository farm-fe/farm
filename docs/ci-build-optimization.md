# CI 编译时间优化方案

> 状态：设计稿（Design Proposal）
> 范围：`.github/workflows/ci.yaml`、`.github/workflows/rust-build.yaml`
> 作者：Farm 维护团队
> 关联问题：CI 端到端耗时过长，PR 反馈慢

## 1. 背景与现状

Farm 的 PR CI 由两个 workflow 组成：

| Workflow | 触发 | 主要 Job |
| --- | --- | --- |
| `ci.yaml` (E2E Tests) | `pull_request` → main | `call-rust-build`、`examples-test`、`ts-test`、`type-check`、`check-*-artifacts` |
| `rust-build.yaml` | 被 `ci.yaml` 通过 `workflow_call` 调用 | 矩阵化的 `build`（12 个 ABI）+ `build-freebsd` |

### 1.1 当前依赖关系

```
                ┌─────────────────────────────────────────┐
                │            call-rust-build              │
                │  (rust-build.yaml, matrix: 12 个 ABI)   │
                └─────────────────────────────────────────┘
                                   │
        ┌──────────────────┬───────┴───────┬───────────────────┐
        ▼                  ▼               ▼                   ▼
  examples-test       ts-test       check-core-artifacts  check-plugin-artifacts
 (matrix: 3 ABI)   (matrix: 3 ABI)
```

`examples-test` 与 `ts-test` 通过 `needs: call-rust-build` 等待整个矩阵完成，
这会让两个相对快速的下游 Job 被最慢的 ABI 拖住。

### 1.2 ABI 矩阵实测耗时（按经验粗排）

| ABI | 用途 | 预计耗时 | 是否被 `examples-test` / `ts-test` 使用 |
| --- | --- | --- | --- |
| `linux-x64-gnu` | examples/ts test、发布 | 中 | ✅ |
| `darwin-arm64`  | examples/ts test、发布 | 中 | ✅ |
| `win32-x64-msvc`| examples/ts test、发布 | 中 | ✅ |
| `linux-x64-musl`| 发布 | 中（docker，启动慢） | ❌ |
| `darwin-x64`    | 发布（cross-compile） | 较慢 | ❌ |
| `win32-ia32-msvc` | 发布（xwin 交叉编译，关 LTO） | **慢** | ❌ |
| `win32-arm64-msvc` | 发布（xwin 交叉编译） | **慢** | ❌ |
| `linux-arm64-musl` | 发布（zig 交叉编译） | 慢 | ❌ |
| `linux-arm64-gnu`  | 发布（zig 交叉编译） | 慢 | ❌ |
| `android-arm-eabi` | 仅 create-farm | 较快 | ❌ |
| `linux-arm-gnueabihf` | 仅 create-farm | 慢 | ❌ |
| `android-arm64`    | 仅 create-farm | 较快 | ❌ |
| `freebsd-x64`      | 仅 create-farm（交叉虚拟机） | **极慢** | ❌ |

**关键观察**：`examples-test` 与 `ts-test` 实际只需要 3 个 ABI 的产物
（`linux-x64-gnu`、`darwin-arm64`、`win32-x64-msvc`），却需要等所有 12+ 个 ABI 都构建完成。

### 1.3 Rust 编译缓存现状

`rust-build.yaml` 中每个矩阵项使用：

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: rust-build-${{ matrix.settings.abi }}
```

每个 ABI 有自己独立的缓存（合理，因 target triple 不同）。
但同一 Job 内的多次 `napi build` 共享 `target/`，理论上应能复用。
然而以下因素导致**实际缓存命中率不高**：

1. **不同 profile 拆分子目录，依赖被重复编译**
   - `packages/core` 用 `--profile release-publish`（`lto = "fat"`、`codegen-units = 1`）。
   - `rust-plugins/{react, sass, replace-dirname}` 用 `--release`。
   - 在 Cargo 中不同 profile 的依赖 crate 会编译两份，因此**整套 swc / rkyv 等重依赖被编译两遍**，
     是单 Job 内编译时间的最大开销。
2. **`lto = "fat"` 让任何对核心 crate 的小改动都触发整图重链**。
   release-publish 的 fat LTO 在 PR 阶段并不需要（PR 只跑测试，不发版）。
3. **没有跨 Job/跨 PR 的二级缓存**（如 sccache + GitHub Actions Cache backend）。
   Swatinem/rust-cache 只缓存 `~/.cargo` 与 `target/`，对 LTO bitcode、
   codegen 单元命中粒度有限。
4. **Docker 构建路径不被 `Swatinem/rust-cache` 缓存**：`linux-x64-gnu` / `linux-x64-musl` 在
   `addnab/docker-run-action` 内执行，主机层挂载了 `~/.cargo/{git,registry}`，但
   `target/` 在容器工作区内，与主机的 rust-cache 仍然能命中（`/build` 即 `${{ github.workspace }}`），
   不过 `CARGO_INCREMENTAL=0` 关掉了增量。
5. **缓存写入互斥**：当前每个 ABI 一个 `shared-key`，没有 `restore-keys` fallback。
   如果某个 ABI 的 toolchain/Cargo.lock 改动，整缓存失效，无法回退到上一个绿 PR 的缓存。

## 2. 优化目标

1. **把 PR 反馈关键路径** —— 即“代码提交 → 收到 examples-test/ts-test 结果”的耗时 —— 缩短到
   3 个用户平台 ABI 完成构建后立即开始，而**不等**任何只用于发布的 ABI。
2. **单 Job 内消除重复 Rust 依赖编译**，让 core 和 3 个 rust 插件的依赖只编译一次。
3. **跨 PR 的二级缓存**进一步降低增量构建时间，并保留 fallback 缓存键以避免冷启动。
4. 不破坏 `release.yaml` / `rust-build.yaml` 既有产物契约（`${{ github.sha }}-${abi}-*` artifact 名称、
   `check-*-artifacts` 检查）。

## 3. 优化方案

方案分为两部分，对应问题陈述里的两个思路。两部分**互不阻塞**，可分阶段落地。

### 3.1 思路一：拆分依赖关系，提升下游并发度

**目标**：让 `examples-test` 与 `ts-test` 各自只等待“自己用到的那个 ABI”构建完成。

#### 方案 A（推荐）：在 `rust-build.yaml` 中按 ABI 分别上传 artifact，并用 job 级 fan-out

GitHub Actions 的 `needs` 是 job 级的，`needs: matrix-job` 会等待整个矩阵。
但**单个矩阵 job 内**只要 artifact 上传完毕，下游就可以下载——只是 `needs` 语法本身做不到“按矩阵实例分别等待”。

解决办法：把 `rust-build.yaml` 中**用户平台**与**纯发布平台**拆成两个独立的可调用 workflow（或两个独立 job）：

```yaml
# .github/workflows/rust-build.yaml
on: workflow_call
jobs:
  build-user-platforms:
    name: Build (User ABI) - ${{ matrix.settings.abi }}
    strategy:
      matrix:
        settings:
          - { os: ubuntu-latest, abi: linux-x64-gnu, ... }
          - { os: macos-latest,  abi: darwin-arm64,  ... }
          - { os: windows-latest, abi: win32-x64-msvc, ... }
    steps: [ ... 同现有 build job ... ]

  build-release-platforms:
    name: Build (Release-only ABI) - ${{ matrix.settings.abi }}
    strategy:
      matrix:
        settings:
          - { abi: linux-x64-musl, ... }
          - { abi: darwin-x64,     ... }
          - { abi: win32-ia32-msvc, ... }
          - { abi: win32-arm64-msvc, ... }
          - { abi: linux-arm64-musl, ... }
          - { abi: linux-arm64-gnu,  ... }
          - { abi: android-arm-eabi, cli_only: true, ... }
          - { abi: linux-arm-gnueabihf, cli_only: true, ... }
          - { abi: android-arm64, cli_only: true, ... }
    steps: [ ... 同现有 build job ... ]

  build-freebsd: { ... 不变 ... }
```

`ci.yaml` 改造：

```yaml
jobs:
  call-rust-build:
    uses: ./.github/workflows/rust-build.yaml

  examples-test:
    # 注意：reusable workflow 作为单一逻辑 job 出现，无法只 needs 其中一部分矩阵。
    # 因此需要在 reusable workflow 中通过 outputs 暴露 build-user-platforms 的状态，
    # 或将 examples-test / ts-test 直接放进 rust-build.yaml 中以 job 级 needs 关联，
    # 或将 reusable workflow 拆成两个文件（见下文 A2，推荐方案）。
    needs: call-rust-build
```

由于 reusable workflow 对外只能整体作为一个“逻辑 job”来 `needs`，
推荐两种落地方式之一：

- **A1. 把 examples-test / ts-test / type-check 也搬进 `rust-build.yaml`**，
  使下游可以 `needs: build-user-platforms`，`check-*-artifacts` 可以 `needs: [build-user-platforms, build-release-platforms]`。
  优点：依赖图最干净。缺点：workflow 文件变大。
- **A2. 不搬动 Job，改为在 `rust-build.yaml` 暴露 outputs**，但 GH 不支持
  reusable workflow 通过 outputs 表达“矩阵子集完成”这一语义；可行的折衷是把
  `rust-build.yaml` 拆成 `rust-build-user.yaml` 和 `rust-build-release.yaml`
  两个文件，`ci.yaml` 中：

  ```yaml
  call-rust-build-user:    { uses: ./.github/workflows/rust-build-user.yaml }
  call-rust-build-release: { uses: ./.github/workflows/rust-build-release.yaml }

  examples-test:           { needs: call-rust-build-user }
  ts-test:                 { needs: call-rust-build-user }
  type-check:              { needs: [] }  # 不依赖任何 build
  check-core-artifacts:    { needs: [call-rust-build-user, call-rust-build-release] }
  check-create-farm-rust-artifacts:
                           { needs: [call-rust-build-user, call-rust-build-release] }
  check-plugin-artifacts:  { needs: [call-rust-build-user, call-rust-build-release] }
  ```

  推荐 **A2**，因为：
  - `release.yaml` 不依赖此拆分（仍可调用任一文件或两者）。
  - 不需要把测试 Job 的实现从 `ci.yaml` 搬出去。
  - 两个 reusable workflow 是天然的“环境一致性边界”，便于未来再细分（例如把
    `linux-x64-gnu` 拎出来作为最快 path）。

#### 方案 B：保持单一 reusable workflow，按 abi 加速 user 平台

更轻量的另一种做法：把 3 个 user-ABI 抬到矩阵的最前面，并把
`fail-fast: false` 保持为 false（已是），同时在 `rust-build.yaml` 中
**让 user-ABI 不依赖任何耗时步骤**（已成立）。
再加一个 job-level concurrency cancel：当 user-ABI 全部成功，取消尚未完成的发布平台
对当前 PR 的运行——但要确保 `release.yaml` 的运行不受影响（仅在 push to tag 时运行）。

不推荐 B：会牺牲 `check-plugin-artifacts` 的覆盖范围（PR 阶段无法发现某个发布平台的 build 回归）。

#### 备注：`type-check` 完全不依赖 native binding

`type-check` 当前并未声明 `needs`，但它会触发自己的 `pnpm --filter @farmfe/cli run build`，
该步骤会**重新走 napi 编译**，与 `call-rust-build` 重复。
建议改为：要么 `needs: call-rust-build-user` 然后 `download-artifact` 注入 binding（更快），
要么把 type-check 改成不需要 binding（mock 出 `binding.cjs`）。

### 3.2 思路二：core 与 plugin build 共享编译缓存

**目标**：在同一个 build job 内，让 `packages/core` + 3 个 rust 插件的 Rust
依赖只编译一次；并通过二级缓存加速跨 PR 的增量构建。

#### 改动 1：统一 PR 阶段的构建 profile（最大收益）

PR 阶段不需要 `release-publish` 的 fat LTO 与 `codegen-units = 1`。
建议在 `Cargo.toml` 中新增一个 `ci` profile：

```toml
[profile.ci]
inherits = "release"
lto = false
codegen-units = 16
debug = false
strip = "debuginfo"
```

并在 `packages/core/package.json` 中新增脚本：

```json
"build:rs:ci": "napi build --platform -p farmfe_node --manifest-path ../../crates/node/Cargo.toml -o binding --js binding.cjs --dts binding.d.ts --profile ci"
```

`rust-build.yaml` 中：

- **PR 触发**（`on: workflow_call` 由 `ci.yaml` 调用）→ 用 `--profile ci`。
- **release 触发** → 维持 `--profile release-publish`。

这样 core 和所有 rust 插件**统一使用同一个 profile**（plugins 改用 `cargo build --profile ci`，
通过 `farm-plugin-tools build --profile ci` 或类似 flag 透传），
Cargo 在同一 `target/ci/` 子目录里只编译一次共享依赖，
共享 swc / rkyv / serde 等大依赖（这些是当前耗时的主要来源）。

落地点：
- `crates/Cargo.toml` 顶层 `[profile.ci]`。
- `packages/core/package.json`：新增 `build:rs:ci`。
- `rust-plugins/*/package.json`：build 脚本支持 `--profile ci`（或新增 `build:ci`）。
- `rust-build.yaml`：在 PR workflow 中调用 `build:ci`。

预期收益：
- 单 Job 内 Rust 依赖**编译次数从 2 次（release + release-publish）降到 1 次**。
- 每个 napi 构建本身因关掉 LTO 也加快 30%–50%。
- 仅 PR 路径受影响，不影响发布产物体积/性能。

#### 改动 2：用 sccache 做二级缓存

在所有 build job 中接入 sccache + GitHub Actions Cache backend：

```yaml
- uses: mozilla-actions/sccache-action@v0.0.5
- run: |
    echo "RUSTC_WRAPPER=sccache"   >> $GITHUB_ENV
    echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV
```

注意事项：
- sccache 与 `CARGO_INCREMENTAL=1` 不兼容，需保持 `CARGO_INCREMENTAL=0`（与现状一致）。
- proc-macro crate 不会被 sccache 缓存——对 napi 来说影响有限。
- 在 docker job（`linux-x64-gnu` / `linux-x64-musl`）中需要把 sccache 二进制和
  `SCCACHE_*` env 透传进容器（`-e SCCACHE_GHA_ENABLED=true -e ACTIONS_CACHE_URL ...
  -v $(which sccache):/usr/local/bin/sccache`）。
- 与 `Swatinem/rust-cache` 并存：rust-cache 仍负责 registry/git 下载缓存，
  sccache 负责对象级编译缓存。

#### 改动 3：保留 cache fallback key

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: rust-build-${{ matrix.settings.abi }}
    restore-keys: |
      rust-build-${{ matrix.settings.abi }}-
```

让 `Cargo.lock` 改动不至于让缓存完全失效，能回退到最近的同 ABI 缓存。

#### 改动 4（可选，长期）：拆分 napi build 与 cargo build

`napi build` 包装层会注入 `--target`、`-Z…` 等参数，不利于自定义 profile。
长期可考虑：
1. 用 `cargo build --profile ci -p farmfe_node …` 直接产出 `.node` 二进制。
2. 再用 `napi build --no-build` 或独立脚本完成 `binding.{js,d.ts}` 生成。
这样 4 个 napi 项目可以共用同一次 `cargo build` 调用（`-p farmfe_node -p farmfe_plugin_react -p
farmfe_plugin_sass -p farmfe_plugin_replace_dirname -p create_farm`），
**单次 cargo invoke 自然共享 `target/` 与 build graph**，比同 profile 多次串行 invoke 进一步省时。

## 4. 落地路线

| 阶段 | 内容 | 风险 | 预期收益 |
| --- | --- | --- | --- |
| P0 | 拆分 `rust-build.yaml` → `rust-build-user.yaml` + `rust-build-release.yaml`；`examples-test` / `ts-test` 只依赖 user | 低，仅工作流改动 | PR 关键路径减去最慢 ABI（≈节省 5–15 min） |
| P1 | 新增 `[profile.ci]`，core + plugins 在 PR 中统一用该 profile | 中，需要 plugin-tools 支持 `--profile`；产物功能不变 | 单 Job 编译时间下降 30–50% |
| P2 | 引入 sccache（GHA backend）+ rust-cache `restore-keys` | 中，需要 docker job 透传 | 跨 PR 二次构建再省 20–40% |
| P3 | 单次 `cargo build` 多 `-p` 出全部 napi crate | 高，需重构 napi 调用方式 | 单 Job 再省 5–15% |

## 5. 验证与回滚

- **指标**：
  - `examples-test` 平均开始时间（`needs` 满足时刻）。
  - `examples-test` / `ts-test` 端到端耗时。
  - `rust-build` 单 job 耗时（按 ABI）。
  - sccache hit rate（通过 `sccache --show-stats`）。
- **回滚策略**：
  - P0 完全是 workflow 层拆分，回滚即恢复单文件。
  - P1 通过 env/profile 名切换；release 路径保持 `release-publish` 不动。
  - P2 通过移除 `RUSTC_WRAPPER=sccache` 即可回滚。
- **不受影响**：
  - `release.yaml` 的发布产物（继续走 `release-publish`）。
  - `check-*-artifacts` 仍验证全 ABI（依赖 release platforms job）。
  - `rust-test.yaml`、`lint.yaml`、`code-spell-check.yaml` 等独立 workflow。

## 6. 开放问题

1. `farm-plugin-tools build` 当前是否原生支持 `--profile <name>`？若不支持需要先在
   `packages/plugin-tools` 中加上参数透传（直接转发到底层 `napi build` /
   `cargo build`）。
2. 是否需要为 `linux-x64-gnu` 在 PR 阶段增加“最快 path”——把它从 docker 中移出，
   只在 release 时走 docker？需评估 musl 兼容矩阵。
3. sccache 在 macos arm64 runner 上的 GHA cache 写入限制（GitHub Actions cache
   总配额 10 GB/repo）需要观察并设置 `SCCACHE_GHA_VERSION` 做隔离。
