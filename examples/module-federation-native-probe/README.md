# Module Federation cross-application E2E

This private example is an automated acceptance suite for the native Farm MF plugin. It uses the repository standalone E2E runner (`e2e.spec.mjs`), not Vitest. No tests are skipped in the supported local matrix.

## Prerequisites

Build the repository's core JS/native packages using the existing source-build instructions, install this workspace's locked dependencies, and install the pinned Playwright Chromium. Build the MF native plugin for the current machine:

```sh
node rust-plugins/module-federation/scripts/build-native.mjs
```

The build script defaults to the local `ci` profile and accepts `--profile`, `--target`, and an optional `--abi` consistency check, plus Cargo arguments such as `--target-dir /absolute/path`. Target-specific output is staged under `npm/darwin-arm64`, `npm/linux-x64-gnu`, or `npm/win32-x64-msvc` as `index.farm`, using Cargo's emitted artifact path. Build the core and plugin from the same checkout, locked dependencies and pinned Rust toolchain, with compatible target/ABI settings. Local `ci` verification uses a `ci`-built core and plugin. Formal CI follows the existing core `release-publish` recipe and plugin `release` recipe; their profile names intentionally differ. For this private plugin, CI uses `--profile release --target <Rust target> --abi <ABI> --locked`. An explicit target still requires the appropriate Rust target and linker. Only macOS arm64 native execution has been verified locally; Linux and Windows require their own native build and acceptance run.

In a COW-managed checkout, keep pnpm/Cargo/browser caches and builds in that checkout’s overlay. Use its tool wrappers and point `PLAYWRIGHT_BROWSERS_PATH` at its matching Playwright browser cache.

## Run

From repository root:

```sh
pnpm run test-e2e -- --example module-federation-native-probe
```

Or run this package's `pnpm run test:e2e`. The suite builds the affected fixtures itself and requires no manually started servers. It does not rebuild Rust or unrelated examples. It reserves loopback ports 4100–4102, 4190 and 4200–4202; occupied ports fail preflight without terminating an existing service. A per-checkout lock rejects overlapping suite runs.

The eleven groups are (the original nine plus a hot-update group for each format):

1. Non-MF CJS/ESM browser regression and unsupported async-require diagnostics.
2. Global production preview: host and two remotes, independent deployment subpaths, static/dynamic imports, CSS/lazy chunks, React/JSX identity, real remote hooks components rendered by host ReactDOM, concurrent loads/init, lazy remote cycle, failures and reload recovery.
3. Global real dev servers: the same assertions plus remote source edit, full-page refresh and source restoration.
4. Rspack ↔ Farm global, including rendered remote hooks components.
5. Webpack ↔ Farm global, including rendered remote hooks components.
6–9. The same production/dev/interoperability groups for optional ESM containers.

Production preview serves the built artifacts on independent static HTTP origins; it deliberately does not use a single Farm preview server. Production and interop harnesses enforce CSP without unsafe-eval/unsafe-inline. Unexpected page errors, console errors and failed requests fail the success-path assertions. Fault-injection pages explicitly test missing entry/expose/JS/CSS and incompatible shared versions.

`evidence-suite.json` contains the grouped results. Per-case browser assertions are recorded in `evidence-*.json`; child stdout/stderr logs are in ignored `e2e-results/`, with repository-runner logs also under root `logs/`.

## Failure, retry and cleanup boundaries

A blocked remote entry prevents application bootstrap from becoming ready. Restoring the endpoint and fully reloading the document must recover and render the remote React component. Same-page automatic retry is not asserted or promised: failed Farm module/ESM loader state is not reset by this test. This describes failed initial loading. Separate hot-update groups now cover already-running applications without reloading.

Each subprocess has a 120-second limit; dev readiness has a 20-second limit. Cleanup stops owned POSIX process groups or, on Windows, the tree of a still-running owned child PID. Graceful suite shutdown restores the edited remote source and releases the suite lock. Once shutdown starts, no further tests or child launches are allowed, including when the repository runner catches a test failure. Cleanup does not search for or kill unrelated processes.

**Windows limitation:** if a direct child exits or crashes before cleanup, its descendants may remain running. The helper does not target an exited PID, which may have been reused, and does not guarantee orphan-tree cleanup. Reliable recovery of that case requires a retained supervisor or Windows Job Object. The simulated Windows tests cover live owned-PID tree termination and graceful IPC shutdown, not recovery of orphaned descendants.

The separately executable interruption check starts the actual suite, waits for its real dev server, requests shutdown using SIGTERM on POSIX or IPC on Windows, and verifies all reserved ports are free, the original source is restored and the lock is gone:

```sh
node examples/module-federation-native-probe/e2e-cleanup-check.mjs
```

Its result is `evidence-e2e-cleanup.json`. It must not run concurrently with the suite.

The lightweight process regressions use real fixture subprocesses, catch-and-continue runner behavior, and an interruption during asynchronous log opening. Run them without native builds:

```sh
node --test examples/module-federation-native-probe/e2e-processes.test.mjs
```

## Locked interoperability matrix

- React / ReactDOM 18.3.1, Playwright Chromium 1.55.0.
- Farm native plugin + official MF runtime 2.9.2.
- Rspack 2.3.0 + MF enhanced 2.9.2.
- Webpack 5.112.2's built-in ModuleFederationPlugin; css-loader 7.1.5 and mini-css-extract-plugin 2.10.2.
- Webpack/Rspack targets explicitly use web + ES2020. ESM remotes use URL module externals; script remotes use globalName@URL.

The enhanced 2.9.2 Webpack adapter was tried but expects `webpack/lib/util/extractUrlAndGlobal`, absent in Webpack 5.112.2. The passing Webpack matrix uses its built-in standard container protocol; it is not a claim that this enhanced/Webpack combination works. Historical fixture directory names contain `rspack`, but both bundlers intentionally consume the same sources.

Static async CJS cycles and synchronous functions requiring async modules remain explicit compile diagnostics. Ordinary synchronous cycles and lazy remote A→B→A are covered. SSR, automatic framework/cross-bundler HMR, manifests/types and arbitrary async cycles are outside this suite.

## Cross-application HMR acceptance (Mac, 2026-11-03)

The same repository command now runs 11 groups. `hot global` and `hot esm` start the real three-app dev setup with `MF_HMR=1`, run `hmr-browser.mjs`, and stop all owned processes. The suite snapshots/restores both remote Button files and the CSS fixture as well as the previous lazy fixture, including signal/timeout cleanup.

The HMR browser test asserts code/CSS updates, shared React identity, host/remote hook state, simultaneous remote updates, compile-error recovery, execution rollback, incompatible-export status, and disconnect/reconnect recovery. `evidence-hmr-global.json` and `evidence-hmr-esm.json` contain actual document tokens, state snapshots, WebSocket URLs, style IDs and statuses. Navigation remains 1 through ordinary updates and failures; the deliberate disconnect produces exactly one explicit reload (navigation count 2), followed by another no-refresh update. Only the intentional disconnect's canceled remote `/__hmr` liveness ping is classified as an expected request failure.

The fixture explicitly accepts remote updates using `bridge.acceptRemote` from `farm:module-federation`. Its stable React component wrappers assume unchanged Hooks structure; general React Refresh signature handling is not claimed. Cross-bundler HMR is not tested. Production loading/CSP and all existing Rspack/Webpack matrix checks remain in the suite. See the bilingual website guide for the opt-in API and limitations.
