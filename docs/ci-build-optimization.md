# CI Compile-Time Optimization Design

> Status: P0–P2 Implemented; P3 Deferred
> Scope: `.github/workflows/ci.yaml`, `.github/workflows/rust-build*.yaml`, `scripts/ci-build-rust-artifacts.sh`
> Author: Farm maintainers
> Related issue: PR CI end-to-end time is too long, slowing down feedback

## 1. Background and Current State

Farm's PR CI consists of two workflows:

| Workflow | Trigger | Main Jobs |
| --- | --- | --- |
| `ci.yaml` (E2E Tests) | `pull_request` → main | `call-rust-build`, `examples-test`, `ts-test`, `type-check`, `check-*-artifacts` |
| `rust-build-user.yaml` / `rust-build-release.yaml` | Invoked by `ci.yaml` via `workflow_call` | Split matrix build jobs for user-tested and release-only ABIs |

### 1.1 Current Dependency Graph

```
                ┌─────────────────────────────────────────┐
                │       call-rust-build-user/release      │
                │       (split rust-build matrices)       │
                └─────────────────────────────────────────┘
                                   │
        ┌──────────────────┬───────┴───────┬───────────────────┐
        ▼                  ▼               ▼                   ▼
  examples-test       ts-test       check-core-artifacts  check-plugin-artifacts
 (matrix: 3 ABIs)   (matrix: 3 ABIs)
```

`examples-test` and `ts-test` declare `needs: call-rust-build`, so they wait for
the **entire** matrix to finish. As a result, these two relatively fast
downstream jobs are blocked by the slowest ABI in the matrix.

### 1.2 ABI Matrix — Approximate Build Time

| ABI | Purpose | Approx. duration | Consumed by `examples-test` / `ts-test`? |
| --- | --- | --- | --- |
| `linux-x64-gnu` | examples/ts test, publish | Medium | ✅ |
| `darwin-arm64`  | examples/ts test, publish | Medium | ✅ |
| `win32-x64-msvc`| examples/ts test, publish | Medium | ✅ |
| `linux-x64-musl`| publish | Medium (docker, slow startup) | ❌ |
| `darwin-x64`    | publish (cross-compile) | Slow | ❌ |
| `win32-ia32-msvc` | publish (xwin cross-compile, LTO off) | **Slow** | ❌ |
| `win32-arm64-msvc` | publish (xwin cross-compile) | **Slow** | ❌ |
| `linux-arm64-musl` | publish (zig cross-compile) | Slow | ❌ |
| `linux-arm64-gnu`  | publish (zig cross-compile) | Slow | ❌ |

**Key observation:** `examples-test` and `ts-test` only need the artifacts for
three ABIs (`linux-x64-gnu`, `darwin-arm64`, `win32-x64-msvc`), so the release-only
ABI builds run in a separate reusable workflow.

### 1.3 Rust Compile Cache — Current Behavior

Each matrix entry in `rust-build.yaml` uses:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: rust-build-${{ matrix.settings.abi }}
```

Each ABI has its own dedicated cache (correct, since the target triple differs).
Within a single job, the multiple `napi build` invocations share `target/` and
in principle should reuse compiled dependencies. In practice, **cache hit rate
is poor** for the following reasons:

1. **Different profiles split into different subdirectories, so dependencies
   are compiled twice.**
   - `packages/core` uses `--profile release-publish` (`lto = "fat"`,
     `codegen-units = 1`).
   - `rust-plugins/{react, sass, replace-dirname}` use `--release`.
   - Cargo compiles each shared dependency once per profile, so heavy crates
     such as the entire **swc** stack, **rkyv**, **serde**, etc. are compiled
     twice. This is the single largest source of wasted time inside a build job.
2. **`lto = "fat"` forces a full re-link on any change to a core crate.**
   The fat-LTO link step is unnecessary for PR builds (PRs only run tests; they
   do not publish artifacts that need maximum runtime performance).
3. **No second-tier cache shared across jobs / PRs** (e.g. sccache backed by
   GitHub Actions Cache). `Swatinem/rust-cache` only caches `~/.cargo` and
   `target/`; its granularity is coarse for LTO bitcode and codegen units.
4. **The docker build paths are partially excluded from `Swatinem/rust-cache`.**
   The `linux-x64-gnu` / `linux-x64-musl` jobs run inside
   `addnab/docker-run-action`. The host mounts `~/.cargo/{git,registry}` into
   the container, but `target/` lives inside `/build`, which equals
   `${{ github.workspace }}` and is therefore still cacheable. However,
   `CARGO_INCREMENTAL=0` is set, disabling incremental compilation entirely.
5. **No cache fallback key.** Each ABI uses a single `shared-key` with no
   `restore-keys`. When `Cargo.lock` or the toolchain changes, the whole cache
   is invalidated and there is no way to fall back to the last green cache.

## 2. Optimization Goals

1. **Shorten the PR critical path** — i.e. the wall-clock time from "push
   commit" to "`examples-test` / `ts-test` results available" — so that those
   jobs start as soon as the 3 user-platform ABIs are ready and do **not** wait
   for any publish-only ABI.
2. **Eliminate duplicate Rust dependency compilation inside a single build
   job**, so core, `create-farm`, and all Rust plugins compile their shared
   dependencies only once.
3. **Add a cross-PR second-tier cache** to reduce incremental build time and
   keep a fallback cache key so that lockfile churn does not force a cold start.
4. Do not change the existing contracts of `release.yaml` / `rust-build.yaml`
   (artifact names of the form `${{ github.sha }}-${abi}-*`, the
   `check-*-artifacts` validations).

## 3. Proposed Design

The implementation has two parts, matching the two angles called out in the
problem statement. P0–P2 are implemented; P3 remains deferred as the optional
long-term restructuring.

### 3.1 Angle 1 — Decouple Downstream Tests to Increase Parallelism

**Goal:** make `examples-test` and `ts-test` wait only for the ABI they
actually consume.

#### Option A (recommended): Split per-ABI artifacts in `rust-build.yaml` and
#### fan out at the job level

GitHub Actions' `needs` operates at the job level. `needs: matrix-job` always
waits for the entire matrix; the language itself cannot express "wait only for
one matrix instance." But a downstream job can begin as soon as the upstream
artifact is uploaded — provided the dependency is expressed at the job level.

The cleanest way to model this is to split `rust-build.yaml` into two
independent reusable workflows (or two independent jobs): one for user
platforms and one for publish-only platforms.

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
    steps: [ ... same as today's build job ... ]

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
    steps: [ ... same as today's build job ... ]
```

`ci.yaml` would then declare:

```yaml
jobs:
  call-rust-build:
    uses: ./.github/workflows/rust-build.yaml

  examples-test:
    # Note: a reusable workflow appears as a single logical job to its caller,
    # so `needs` cannot target a subset of its matrix. To make this work we
    # must either (a) expose outputs from build-user-platforms in the reusable
    # workflow, (b) move examples-test / ts-test into rust-build.yaml so they
    # can `needs: build-user-platforms` at the job level, or (c) split the
    # reusable workflow into two files (see Option A2 below — recommended).
    needs: call-rust-build
```

Two practical landing options:

- **A1. Move `examples-test` / `ts-test` / `type-check` into `rust-build.yaml`**,
  so they can `needs: build-user-platforms`, and
  `check-*-artifacts` can `needs: [build-user-platforms, build-release-platforms]`.
  Pros: cleanest dependency graph. Cons: the workflow file grows large.
- **A2. Expose `outputs` from `rust-build.yaml`** — but GitHub Actions does
  not allow reusable workflows to express "a subset of the matrix has
  completed" via outputs. The pragmatic workaround is to split
  `rust-build.yaml` into two files,
  `rust-build-user.yaml` and `rust-build-release.yaml`, and have `ci.yaml`
  call both:

  ```yaml
  call-rust-build-user:    { uses: ./.github/workflows/rust-build-user.yaml }
  call-rust-build-release: { uses: ./.github/workflows/rust-build-release.yaml }

  examples-test:           { needs: call-rust-build-user }
  ts-test:                 { needs: call-rust-build-user }
  type-check:              { needs: [] }  # does not need any build
  check-core-artifacts:    { needs: [call-rust-build-user, call-rust-build-release] }
  check-create-farm-rust-artifacts:
                           { needs: [call-rust-build-user, call-rust-build-release] }
  check-plugin-artifacts:  { needs: [call-rust-build-user, call-rust-build-release] }
  ```

  **A2 is recommended** because:
  - `release.yaml` is unaffected (it can keep calling either file or both).
  - We do not have to relocate the test jobs out of `ci.yaml`.
  - The two reusable workflows form a natural environment boundary for future
    refinement (e.g. a dedicated fast `linux-x64-gnu` path).
  - The split workflows must use distinct concurrency groups; reusable workflows
    can inherit the caller workflow name in `${{ github.workflow }}`, so hardcode
    a `rust-build-user-*` / `rust-build-release-*` prefix to avoid one split
    cancelling the other.
  - The split workflows intentionally avoid sccache/GitHub Actions sccache
    storage; stale or cross-target compiler cache entries can make CI failures
    harder to diagnose across the target matrix.

#### Option B: Keep a single reusable workflow and reorder the matrix

A lighter-weight alternative: hoist the 3 user ABIs to the front of the matrix,
keep `fail-fast: false` (already true), and make sure user ABIs do not depend
on any slow setup step (already true). Optionally, add a concurrency-cancel
rule to abort still-running publish ABIs once the 3 user ABIs succeed — being
careful not to break `release.yaml`, which only runs on tag pushes.

**Not recommended.** This would reduce coverage of `check-plugin-artifacts`
during PRs, hiding regressions in publish-only platforms until release time.

#### Side note: `type-check` does not need the native binding

`type-check` currently declares no `needs`, but it runs
`pnpm --filter @farmfe/cli run build`, which triggers a full napi compile —
duplicating work already done by `call-rust-build`. Either:
- Set `needs: call-rust-build-user` and inject a downloaded binding via
  `download-artifact` (faster), or
- Refactor `type-check` to not need the binding at all (mock `binding.cjs`).

### 3.2 Angle 2 — Share Compile Cache Across Core and Plugins

**Goal:** within a single build job, make `packages/core`, `packages/create-farm`,
and all Rust plugins compile their shared Rust dependencies exactly once, and
add a second-tier cache to speed up incremental builds across PRs.

#### Change 1 — Unify the PR build profile (biggest win)

PRs do not need `release-publish`'s fat LTO or `codegen-units = 1`. Add a `ci`
profile to the root `Cargo.toml`:

```toml
[profile.ci]
inherits = "release"
lto = false
codegen-units = 16
debug = false
strip = "debuginfo"
```

Add a new script in `packages/core/package.json`:

```json
"build:rs:ci": "napi build --platform -p farmfe_node --manifest-path ../../crates/node/Cargo.toml -o binding --js binding.cjs --dts binding.d.ts --profile ci"
```

In `rust-build.yaml`:

- **PR trigger** (invoked via `workflow_call` from `ci.yaml`) → use `--profile ci`.
- **Release trigger** → keep `--profile release-publish`.

This way core, `create-farm`, and every Rust plugin use the **same profile**, so Cargo places
all shared dependencies under one `target/ci/` subdirectory and compiles them
once. Heavy shared dependencies (swc, rkyv, serde, …) — which dominate today's
job time — are no longer compiled twice.

Concrete touch points:
- Top-level `Cargo.toml`: add `[profile.ci]`.
- `packages/core/package.json`: add `build:rs:ci`.
- `packages/create-farm/package.json` and `rust-plugins/*/package.json`: build
  scripts accept `--profile ci` (or add a `build:ci` script).
- `rust-build.yaml`: PR path calls the `build:ci` variant; `create-farm` and
  plugins build with `--profile ci`.

Expected gains:
- Number of times shared Rust deps are compiled within a job drops from
  **2 → 1**.
- Each napi build itself is 30–50% faster thanks to disabling LTO.
- Only the PR path is affected; published artifact size and runtime are
  unchanged.

#### Change 2 — Add sccache as a second-tier cache

Wire sccache backed by GitHub Actions Cache into every build job:

```yaml
- uses: mozilla-actions/sccache-action@v0.0.5
- run: |
    echo "RUSTC_WRAPPER=sccache"   >> $GITHUB_ENV
    echo "SCCACHE_GHA_ENABLED=true" >> $GITHUB_ENV
```

Caveats:
- sccache is incompatible with `CARGO_INCREMENTAL=1`; keep `CARGO_INCREMENTAL=0`
  (already the case).
- proc-macro crates are not cached by sccache — limited impact for napi.
- Docker jobs (`linux-x64-gnu` / `linux-x64-musl`) must propagate the sccache
  binary and `SCCACHE_*` env vars into the container, e.g.
  `-e SCCACHE_GHA_ENABLED=true -e ACTIONS_CACHE_URL=... -v $(which sccache):/usr/local/bin/sccache`.
- Coexists with `Swatinem/rust-cache`: rust-cache continues to cache the
  registry / git downloads, while sccache covers object-level compilation
  results.

#### Change 3 — Keep a cache fallback prefix

`Swatinem/rust-cache@v2` does not expose the same `restore-keys` input as
`actions/cache`, so the workflows use an ABI/profile-specific `prefix-key` plus
the stable ABI `shared-key` supported by that action:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    prefix-key: rust-build-${{ inputs.profile }}-${{ matrix.settings.abi }}
    shared-key: rust-build-${{ matrix.settings.abi }}
```

This keeps cache entries separated by build profile while still using the
action-supported fallback prefix behavior for the same ABI.

#### Change 4 (optional, long term) — Decouple `napi build` from `cargo build`

The `napi build` wrapper injects `--target`, `-Z…` and other flags, which makes
custom profiles awkward. Longer term, consider:
1. Run `cargo build --profile ci -p farmfe_node …` directly to produce the
   `.node` binary.
2. Use `napi build --no-build` (or a small standalone script) to emit
   `binding.{js,d.ts}`.

This lets the core binding, `create-farm`, and plugin bindings share a single
`cargo build` invocation (`-p farmfe_node -p create_farm -p ...`), so
`target/` and the build graph are shared natively, saving even more time than
serial same-profile invocations.

## 4. Rollout Plan

| Phase | Content | Risk | Expected gain |
| --- | --- | --- | --- |
| P0a | ✅ Extract the common Rust artifact build steps into `scripts/ci-build-rust-artifacts.sh` so the user-platform and release-platform workflows cannot drift. | Low — mechanical workflow refactor | Safer split with no behavior change. |
| P0b | ✅ Split `rust-build.yaml` into `rust-build-user.yaml` (`linux-x64-gnu`, `darwin-arm64`, `win32-x64-msvc`) and `rust-build-release.yaml` (publish-only and CLI-only ABIs, including FreeBSD). `rust-build.yaml` remains as a release-compatible wrapper. | Low — workflow files only | Establishes job-level dependencies for the fast path. |
| P0c | ✅ Update `ci.yaml` so `examples-test` and `ts-test` depend only on `call-rust-build-user`, while `check-core-artifacts`, `check-create-farm-rust-artifacts`, and `check-plugin-artifacts` depend on both user and release builds. | Low — dependency graph only | PR critical path no longer includes the slowest ABI (≈ 5–15 min saved). |
| P1 | ✅ Add `[profile.ci]`; `packages/core`, `packages/create-farm`, and all Rust plugins use it on the PR path. `farm-plugin-tools build` forwards `--profile` to napi and now removes the script-level `--release` flag when a profile is supplied. | Medium — plugin-tools and create-farm scripts must accept `--profile`; artifact behavior unchanged | 30–50% reduction in single-job build time. |
| P2 | ✅ Add sccache (GHA backend) and rust-cache fallback prefixes; propagate the sccache binary and cache env into docker builds. | Medium — docker jobs must receive the binary and cache env | 20–40% further reduction on warm runs. |
| P3 | Deferred: replace serial `napi build` invocations with one multi-package `cargo build`, then emit napi JS/DTS wrappers separately. | High — requires restructuring how napi is invoked | 5–15% additional saving per job. |

P0–P2 are now implemented together because P1/P2 depend on the shared build
script introduced by P0a. Keep P3 as a long-term cleanup after the faster PR
path is stable and CI metrics confirm the split is working as expected.

## 5. Validation and Rollback

- **Metrics:**
  - Time at which `examples-test` becomes runnable (i.e. its `needs` are
    satisfied).
  - End-to-end duration of `examples-test` / `ts-test`.
  - Duration of each `rust-build` matrix job (per ABI).
  - sccache hit rate (via `sccache --show-stats`).
- **Rollback strategy:**
  - P0 is a pure workflow refactor; reverting it restores the single file.
  - P1 is gated on a profile name / env var; the release path keeps
    `release-publish` untouched.
  - P2 can be disabled by removing `RUSTC_WRAPPER=sccache`.
- **Not affected:**
  - `release.yaml` publish artifacts (still built with `release-publish`).
  - `check-*-artifacts` still cover every ABI (via the release-platforms job).
  - Independent workflows: `rust-test.yaml`, `lint.yaml`,
    `code-spell-check.yaml`, etc.

## 6. Open Questions

1. Answered: `@napi-rs/cli` supports `--profile`; `farm-plugin-tools build`
   already forwards unknown args, and now explicitly removes the default
   `--release` flag when `--profile <name>` is supplied so Cargo receives a
   single profile selector.
2. Should we add an even faster "hot path" by moving `linux-x64-gnu` off
   docker on the PR path and running it directly on the host, only using
   docker for releases? Needs an evaluation of musl/glibc compatibility.
3. sccache writes against the GitHub Actions cache quota (10 GB per repo).
   We should monitor usage and possibly set `SCCACHE_GHA_VERSION` for
   isolation between branches.
