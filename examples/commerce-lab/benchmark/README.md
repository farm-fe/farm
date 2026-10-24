# Benchmark tooling

The benchmark protocol is defined in [the comparison design](../../../docs/v2-ecosystem-comparison-design.md). Complete A/B/C correctness acceptance before collecting publishable performance samples.

`statistics.mjs` provides pure aggregation for raw records. Cells are separated by group, scenario, scale and cache mode. Successful durations must be finite and nonnegative; failed samples count toward the failure rate and remain in raw CSV, but never enter successful timing quantiles. Empty successful cells have no timing result.

Quantiles use linear interpolation at `(n - 1) * p`. Reports include median, Q1, Q3, IQR, P95 and min/max. `sufficientSamples` requires 30 successful samples for HMR scenarios and 10 for other scenarios; it is only a sample-count flag, not a correctness or release gate.

`node benchmark/environment.mjs` captures CPU, memory, OS, Node/pnpm/Rust, repository commit, dirty working-tree status, lockfile SHA-256 and source package versions. It also resolves the core native loader and Rust plugin loader paths, recording real paths, byte counts and SHA-256 for core plus 13 plugins. Cargo.lock SHA-256 and selected Rust parser/compiler dependency versions are recorded separately: lockfile resolution cannot prove which dependencies are embedded in an existing native binary. Build sampling fingerprints native files again afterward and fails the run if any selected path or hash changed. Building unrelated plugins during sampling therefore invalidates the run.

`node benchmark/artifacts.mjs js` (or `rust` / `mixed`) verifies the built directory: every eligible resource at least 1024 bytes has gzip output, every gzip has an original, and decompression matches the original bytes. It reports byte totals for the verified pairs. This does not establish which files a browser actually requests or the performance of transfer compression.

`node benchmark/build-sampler.mjs` runs 10 seeded rounds by default. Each round shuffles JS/Rust/mixed order, then runs each group's cold build followed by its warm build in fresh Node processes. Each cold sample uses a unique initially absent cache and output directory; warm uses the matching successful cold cache. No existing directories are deleted. Set `COMMERCE_ROUNDS=1` for a smoke test or `COMMERCE_SEED` to reproduce order.

Each run saves environment metadata, its schedule, per-build logs, incrementally updated raw JSON, raw CSV and a statistical summary under `reports/build-*/`. End-to-end duration covers process launch through close; gzip and Worker artifact checks happen afterward. Failed cold builds prevent the corresponding warm sample from being treated as valid. The timeout is 10 minutes. Windows termination uses taskkill /T against the owned child process; failures are recorded and fall back to direct-child termination. Other platforms currently terminate only the direct child. Reports explicitly set `publishable: false` while the business workload and complete capability acceptance remain incomplete.

Set `COMMERCE_CAPTURE_MEMORY=1` for a separate Windows instrumented run. This captures the root and descendant WorkingSetSize sum, excluding the browser. Shared pages may be counted more than once. Each raw observation records collection duration and timestamp; requested pause is 200 ms between completed collections, not a guaranteed 200 ms sampling rate. Short-lived processes and peaks can be missed. Missing root observations produce null, and no valid observations fail the sample rather than report zero. Instrumented timing uses a distinct scenario and must not be mixed into uninstrumented build timing. `node tests/process-tree-live.mjs` verifies a real owned process tree and confirms descendants have exited after termination.

Non-Windows process-tree memory measurement, faster Windows peak capture, repeated HMR sampling and the scale generator are still outstanding. There are no published performance measurements in this directory.

`node tests/hmr-capabilities.mjs` performs six browser preflight checks (leaf, shared component, Sass dependency, newly added Tailwind class, SVG and MDX). Set `COMMERCE_URL`, `COMMERCE_GROUP` and `COMMERCE_HMR_OUTPUT` to select the running development server and save raw JSON. Each edit is restored in `finally`, and restoration failures remain failures. These single preflight observations are not benchmark samples; repeated sampling, run provenance and process-tree memory capture remain outstanding.

`node benchmark/hmr-sampler.mjs` defaults to 30 seeded rounds across JS/Rust/mixed. Only one server runs at a time so edits cannot trigger competing group compilers. Each group/round starts a fresh server session, runs all six warmup edits, then measures six edits; persistent cache is retained per group. Startup and warmup are excluded from measured durations. This protocol measures repeated edits in fresh sessions, not a single long-lived 30-edit session. Set `COMMERCE_ROUNDS=1` for smoke verification. Reports include each measured and warmup JSON/log, server logs, raw CSV, summary, and native fingerprints before/after. Every successful edit must preserve the document, preserve interaction state where applicable, and restore source. The parent also restores recognized probe edits after driver termination; unexpected concurrent source changes cause failure instead of being overwritten.

`node --test tests/source-recovery.test.mjs` checks recognized probe restoration, concurrent-edit preservation, missing-file recovery and path guards. `node tests/source-recovery-live.mjs` kills a real owned Windows driver after its edit and confirms parent recovery restores the original bytes in a temporary workspace. Later HMR runs save per-driver recovery JSON. Freeze benchmark code and avoid unrelated work during publishable sampling; development-time smoke/calibration runs do not establish controlled-machine performance.

Run the statistics regression tests from the example directory:

```powershell
node --test tests/statistics.test.mjs
```
