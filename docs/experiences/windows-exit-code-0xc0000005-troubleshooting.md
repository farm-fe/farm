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

## Follow-up Recommendations

- Add repeated Windows build stability checks (at least 10 runs) for high-risk plugins.
- Add lightweight hook-level debug logs behind a switch.
- Standardize failure reports with: exit code + last active hook + resource path.