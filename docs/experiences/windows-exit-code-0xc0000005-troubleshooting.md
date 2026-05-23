# Windows Exit Code Troubleshooting (-1073741819 / 0xC0000005)

## Scope

This document summarizes practical debugging and remediation experience for Rust plugin related build failures on Windows that exit with `-1073741819` (hex `0xC0000005`, Access Violation).

Primary goals:
- Narrow the issue down to one plugin, one hook, and one resource path.
- Fix it with minimal changes and avoid introducing new lock/lifecycle risks.

## Typical Symptoms

- Build logs show "Build completed", but the process still exits with `-1073741819`.
- The issue appears only in some Rust plugins, often intermittently.
- Re-running the same command alternates between success and failure.
- Reproduction is easier in examples involving assets, cache, or concurrency.

## Conclusions From This Investigation

### 1. `rust-plugins/icons`: Disk cache implementation was the key risk

Observed pattern:
- Random Windows crashes.
- Strong correlation with paths involving `cached::DiskCache`.

Effective fix:
- Replace `cached::DiskCache` with a simple file-based cache (key hash -> local file).
- Treat cache read/write failures as non-fatal and keep the main flow running.
- Avoid complex retry + sleep backoff during cache init to reduce Windows handle contention.

Takeaways:
- For optional performance features like cache, prioritize graceful degradation.
- On Windows, reduce dependence on file-lock-sensitive cache internals.

### 2. `rust-plugins/url`: Root cause was teardown drop order, not emit lock or params address

Observed pattern:
- Build output could show "Build completed", but process still exited with `-1073741819`.
- Instrumentation showed `emit_file` insert and lock release both completed successfully.
- Crash occurred in process teardown.

Rejected hypotheses:
- Not lock release in `emit_file`.
- Not `EmitFileParams` memory address ownership.

Confirmed root cause:
- `CompilationContext` dropped `plugin_driver` before `resources_map`.
- `plugin_driver` drop unloads Rust plugin DLL.
- `resources_map` drop later freed resource entries while DLL-dependent code/data was already gone.
- This produced Access Violation during teardown.

Effective fix:
- Reorder fields in `CompilationContext` so `resources_map` is declared before `plugin_driver`.
- Rust drops fields in declaration order, so resources are released before plugin DLL unload.

Validation:
- Rebuilt native core binding and plugin.
- Re-ran `examples/rust-plugin-url` build multiple times.
- Stable exit code `0` and correct output artifacts.

### 3. `rust-plugins/wasm`: keep `emit_file`, avoid Windows plugin-cache teardown risk

Observed pattern:
- `examples/rust-plugin-wasm` could print build success and then exit with `-1073741819`.
- Crash happened after normal emit/write output stages.

Remediation guideline:
- Do not replace wasm emission with forced data URL inline mode as a crash workaround.
- Keep `emit_file` behavior unchanged for wasm assets.
- Prefer mitigating teardown-sensitive paths in plugin cache lifecycle on Windows (for example, disable plugin-level cache load/write hooks for this plugin).

### 4. `rust-plugins/compress`: keep parallelism, but do not use plugin-side Rayon iterators

Observed pattern:
- `examples/rust-plugin-compress` could print successful build output and still exit with `-1073741819`.
- Replacing parallel compression with serial iteration stabilized the build, confirming the issue was in the parallel execution path rather than compressed output contents.
- A later attempt using `context.thread_pool.install(...)` plus plugin-side Rayon iterators triggered a Rayon registry assertion on Windows instead of the original access violation.

Rejected approaches:
- Do not parallelize directly over `resources_map` with `par_iter()`.
- Do not call `context.thread_pool.install(...)` and then use plugin-side `into_par_iter()` for this plugin on Windows.

Effective fix:
- Build an owned list of compression tasks first, separate from `resources_map` mutation.
- Use `std::mem::take(&mut resource.bytes)` to move original bytes out of each resource without cloning the whole buffer.
- Schedule compression work with `context.thread_pool.spawn(...)`.
- Collect results through a standard channel.
- Restore original bytes back into `resources_map` before inserting compressed artifacts.
- Perform final `resources_map` mutation serially.

Why this worked:
- It keeps parallel compression.
- It avoids plugin-side Rayon iterator / registry interactions that were unstable on Windows in this DLL-backed plugin path.
- It removes unnecessary large-buffer cloning during task collection.

Validation:
- Rebuilt `@farmfe/plugin-compress`.
- Re-ran `node scripts/test-examples.mjs --skip-build-js-plugins --example rust-plugin-compress` repeatedly.
- Stable exit code `0` with no panic and no post-build crash.

### 5. `rust-plugins/worker`: DLL pinning is the only reliable fix for Rayon TLS / host thread lifetime

Observed pattern:
- `examples/rust-plugin-worker` printed "Build completed" then exited with `-1073741819`.
- Crash rate was ~70% (7 out of 10 runs) and fully reproducible.
- No crash occurred inside the build itself — only during process teardown.

Root cause analysis:

The worker plugin's `load()` and `transform()` hooks are invoked from **host Rayon pool threads** (i.e., threads owned by `CompilationContext::thread_pool`).  When those threads execute DLL code, any DLL-statically-linked thread-local storage (TLS) — from `rayon`, `parking_lot`, `std`, SWC, etc. — is initialised on those threads.

The teardown sequence is:
1. `CompilationContext` drops field by field in declaration order.
2. `thread_pool` drops → Rayon sends a non-blocking terminate signal to worker threads (does not wait).
3. `plugin_driver` drops → calls `FreeLibrary(worker_plugin.node)` → DLL unloaded.
4. Host Rayon threads eventually exit → their TLS destructors fire at DLL addresses → `STATUS_ACCESS_VIOLATION`.

Rejected approaches tried in this investigation:

| Approach | Why it failed |
|---|---|
| Reorder `resources_map` before `plugin_driver` (already done for url/compress) | Necessary but not sufficient — the crash is in host thread TLS, not in resource memory |
| Share host `ThreadPool` with nested `Compiler` | Host threads still acquire DLL TLS during `load()` calls; sharing the pool doesn't change that |
| Spawn isolated `std::thread::spawn + join` for nested compilation | Only protects nested pool threads, not the host Rayon threads that called `load()` |
| Custom `spawn_handler` with counter + `Condvar` to drain nested pool | Same limitation — host threads are not in scope |
| Global serial mutex | Reduces frequency; cannot eliminate the race between thread exit and DLL unload |

Effective fix — **DLL pinning via `GetModuleHandleExA` with `GET_MODULE_HANDLE_EX_FLAG_PIN`**:

```rust
#[cfg(target_os = "windows")]
fn pin_current_dll_in_memory() {
    use std::sync::OnceLock;
    static PINNED: OnceLock<()> = OnceLock::new();
    PINNED.get_or_init(|| {
        const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x00000004;
        const GET_MODULE_HANDLE_EX_FLAG_PIN: u32 = 0x00000001;
        extern "system" {
            fn GetModuleHandleExA(flags: u32, name: *const u8, phm: *mut *mut u8) -> i32;
        }
        let mut handle: *mut u8 = std::ptr::null_mut();
        unsafe {
            GetModuleHandleExA(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
                pin_current_dll_in_memory as *const u8,
                &mut handle,
            );
        }
    });
}
```

Call this once from `FarmfePluginWorker::new()` on Windows.

Why this is the correct fix:
- `GET_MODULE_HANDLE_EX_FLAG_PIN` increments the DLL's reference count permanently.  Subsequent `FreeLibrary` calls can never decrement it to zero — the DLL stays resident for the entire process lifetime.
- All host Rayon threads can safely run their TLS destructors against DLL addresses at any time, regardless of when `plugin_driver` calls `FreeLibrary`.
- This is the **canonical Windows pattern** for DLLs that install thread-lifetime hooks or whose code may be referenced by threads whose lifetime extends beyond explicit unloads.
- No functional regression: the DLL is already present in memory (the process is about to exit anyway); pinning only affects the unload count, not memory consumption.

Why this works while the other approaches don't:
- The crash originates on **host pool threads** that called into DLL code during the build.  No amount of nested-compiler lifecycle management can prevent host threads from acquiring DLL TLS.  The only correct fix is to ensure the DLL remains valid for as long as those threads live — which is what pinning achieves.

Validation:
- Rebuilt `@farmfe/plugin-worker` (`pnpm --filter @farmfe/plugin-worker run build`).
- Ran `node scripts/test-examples.mjs --skip-build-js-plugins --example rust-plugin-worker` 10 times.
- **10/10 exit 0**, no access violations.

#### Architectural improvement: deferred compilation via `build_end()` / `update_finished()`

After the DLL-pinning fix was validated, the plugin was further refactored to move `Compiler::new()` + `compile()` out of Rayon-dispatched hooks entirely for non-inline workers.

**Before the refactoring:**
- `load()` and `transform()` hooks (both Rayon threads) called `build_worker()` which internally ran `Compiler::new()` + `compile()`.

**After the refactoring:**
- `load()` for **non-inline** workers: computes the output URL deterministically (no compilation), pushes to a `pending_workers: Mutex<Vec<PendingWorker>>`, returns the URL wrapper.
- `transform()` for `new Worker(new URL(...))` patterns: computes the URL deterministically, defers to `pending_workers`, transforms the source.
- `build_end()` hook (main thread): drains `pending_workers`, calls `build_worker()` for each, emits files.
- `update_finished()` hook (main thread): same as `build_end()`, for HMR incremental updates.
- `load()` for **inline** workers still compiles immediately (bytes are needed in the returned module code).

DLL pinning is still necessary because `load()` and `transform()` themselves run on host Rayon threads and acquire DLL TLS, even without calling `Compiler::new()`.  Pinning ensures those threads can safely run TLS destructors after `FreeLibrary`.

**Key hooks used (both new to the worker plugin):**

| Hook | Thread | Purpose |
|---|---|---|
| `build_end()` | Main | Compile + emit all pending non-inline workers after the build graph is complete |
| `update_finished()` | Main | Same, after each HMR incremental rebuild |

### 6. General principle: parallel Rust plugin work on Windows

If a Rust plugin spawns threads (directly or via Rayon) that execute DLL code, those threads acquire DLL-linked TLS.  On Windows, `FreeLibrary` does not wait for threads to exit.  If those threads outlive the `FreeLibrary` call, their TLS destructors will crash.

Decision tree:
- Does the plugin's hook run on a host Rayon thread? → **Always pin the DLL** (`GET_MODULE_HANDLE_EX_FLAG_PIN`).
- Does the plugin create its own Rayon pool for parallel work? → Also drain the pool (wait for all workers to exit) before the hook returns — *or* pin the DLL.
- Does the plugin use `context.thread_pool.spawn()`? → Those are host threads; DLL pinning is still needed if DLL TLS is acquired.

## Reusable Debug Workflow

1. Minimize reproduction scope.
- Start with one example (for example, `examples/rust-plugin-url`).
- Do not start from the full examples suite.

2. Separate example failure from test-orchestration failure.
- Run the example build directly first.
- Then run `scripts/test-examples.mjs` and check if unrelated dependencies fail.

3. Stage suspected crash points.
- Add before/after markers around lock, insert, write, and teardown-adjacent paths.
- Confirm whether crash happens in runtime flow or teardown flow.

4. Run repeated stability checks.
- Run the same build 10 to 20 times.
- Use failure rate, not one-off outcome.

5. For dynamic-library plugin systems.
- Always inspect lifecycle and field drop ordering on host-side owner structs.
6. For parallel Rust plugin work on Windows.
- Prefer `thread_pool.spawn(...)` on owned tasks over plugin-side Rayon iterators when the plugin runs from a dynamically loaded library.
- If task inputs contain large `Vec<u8>`, prefer ownership transfer (`mem::take`) over full-buffer clone.

## Validation Templates (Windows)

### A. Single example build validation

Run from repository root:

```powershell
Set-Location -Path "C:\Users\lj_bright\Desktop\code\farm\examples\rust-plugin-url"
pnpm farm build
```

Pass criteria:
- Output includes "Build completed".
- `dist` artifacts are complete.
- Exit code is consistently 0. If not, inspect post-build lifecycle steps.

### B. Repeated stability check (5-second threshold)

```powershell
Set-Location -Path "C:\Users\lj_bright\Desktop\code\farm\examples\rust-plugin-url"
1..10 | ForEach-Object {
	Write-Host "RUN $_"
	$p = Start-Process -FilePath "pnpm" -ArgumentList "farm","build" -NoNewWindow -PassThru
	if (-not $p.WaitForExit(5000)) {
		Write-Host "TIMEOUT(>5s) at run $_"
		try { Stop-Process -Id $p.Id -Force } catch {}
	} else {
		Write-Host "EXIT=$($p.ExitCode)"
	}
}
```

### C. Orchestration script validation (after single-example stability)

```powershell
Set-Location -Path "C:\Users\lj_bright\Desktop\code\farm"
node scripts/test-examples.mjs --skip-build-js-plugins --from rust-plugin-url --to rust-plugin-url
```

If this fails:
- Check whether the script is failing in unrelated dependency builds.
- Do not immediately attribute failure to `rust-plugin-url` itself.

## Common Misdiagnoses

- Misdiagnosis 1: "Build completed" means success.
	- Correction: Exit code must also be 0.

- Misdiagnosis 2: `test-examples` failure means target example failure.
	- Correction: The script may fail in a different package on the dependency path.

- Misdiagnosis 3: Intermittent Windows crash must be a concurrency issue.
	- Correction: Cache internals, handle contention, hook timing, and teardown order can all trigger similar exit codes.

## Change Principles For Similar Issues

- Prefer minimal structural simplification first.
- Avoid shared mutable state across lifecycle phases.
- Design cache/copy features as non-critical and degradable.
- In DLL-based plugin systems, treat drop order as a correctness constraint.
- Do not "fix" Windows native crashes by bypassing `emit_file` (for example, forcing data URL inline only);
	keep `emit_file` semantics intact and fix host/plugin lifecycle issues (drop order, teardown timing, cache ownership).

## Follow-up Recommendations

- Current Windows example sweep status (all stable, 10/10 exit 0):
	- `rust-plugin-auto-import-react`, `rust-plugin-auto-import-vue`
	- `rust-plugin-compress` (parallel compression via `thread_pool.spawn` + `mem::take`)
	- `rust-plugin-icons-react`, `rust-plugin-icons-vue`
	- `rust-plugin-image`, `rust-plugin-modular-import`, `rust-plugin-react-components`
	- `rust-plugin-strip`, `rust-plugin-svgr`, `rust-plugin-url`, `rust-plugin-virtual`
	- `rust-plugin-wasm` (fixed: static URL string instead of `?url` import)
	- `rust-plugin-worker` (fixed: DLL pinning via `GetModuleHandleExA` + `GET_MODULE_HANDLE_EX_FLAG_PIN`)

- Add repeated Windows build stability checks (at least 10 runs) for high-risk plugins.
- Add lightweight hook-level debug logs behind a switch.
- Standardize failure reports with: exit code + last active hook + resource path.