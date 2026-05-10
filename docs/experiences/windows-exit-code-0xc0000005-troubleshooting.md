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

- Current Windows example sweep status:
	- Stable with exit code 0 through `scripts/test-examples`: `rust-plugin-auto-import-react`, `rust-plugin-auto-import-vue`, `rust-plugin-compress`, `rust-plugin-icons-react`, `rust-plugin-icons-vue`, `rust-plugin-image`, `rust-plugin-modular-import`, `rust-plugin-react-components`, `rust-plugin-strip`, `rust-plugin-svgr`, `rust-plugin-url`, `rust-plugin-virtual`.
	- `rust-plugin-wasm` currently fails with a module resolution error for `json_typegen_wasm_bg.wasm?url`; this is not the same as a post-build native crash.
	- `rust-plugin-worker` is the remaining reproduced Windows native crash case in the current sweep (`0xC0000005` after successful build output).

- Add repeated Windows build stability checks (at least 10 runs) for high-risk plugins.
- Add lightweight hook-level debug logs behind a switch.
- Standardize failure reports with: exit code + last active hook + resource path.