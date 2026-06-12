<!-- cspell:ignore cranelift cranelift's doctest soruce recieve -->

# Emotion Deadlock Investigation & Companion Test/Spell Fixes

This note captures the diagnosis and fix process for the `examples/emotion`
build hang investigated in 2026-06, plus the playbook used to fix the
`svgr-rs` cargo tests and the repo-wide cspell errors that came along with
it. Keep it as a quick reference for similar issues.

## 1. Emotion Build Hang — Two-Layer Root Cause

### 1.1 Symptoms

- `examples/emotion` intermittently hangs during build, both locally
  (especially with `FARM_THREAD_NUMS=2`) and on GitHub CI. The process
  stalls inside the SWC wasm plugin transform stage.
- Setting `persistentCache: false` does not change the behavior, so it is
  unrelated to the cache.

### 1.2 Diagnosis

1. **Stack sampling on macOS (`sample`)**: Sampling the hung `node` process
   showed multiple rayon workers blocked on:
   - the `wasmer` internal mutex (the global lock of the plugin runtime
     singleton);
   - `cranelift`'s `Registry::in_worker_cross`, which waits for another
     worker in the same rayon pool to finish compilation.
2. **Diff regression**: `git diff origin/main` pointed to the SWC wasm
   plugin transform entrypoint inside `process_module` at
   [crates/plugin_script/src/lib.rs](file:///Users/bytedance/Desktop/opensource/farm-v2/crates/plugin_script/src/lib.rs).
3. **Two-layer root cause**:
   - **Layer 1 (wasmer/cranelift)**: wasmer holds a global mutex while
     instantiating concurrently; cranelift's `in_worker_cross` then asks
     the same rayon pool for another worker, producing the classic
     "worker A holds the lock and waits for worker B; worker B waits for
     the lock held by A" nested starvation.
   - **Layer 2 (application side)**: Farm calls the wasm transform
     directly inside a rayon `par_iter`, which amplifies the starvation
     into a reliable hang.

### 1.3 Fix

Move the SWC wasm plugin's `try_with` + `transform_by_swc_plugins` block
out of any rayon worker and run it synchronously on a dedicated OS thread
via `std::thread::scope`:

```rust
// crates/plugin_script/src/lib.rs (excerpt)
std::thread::scope(|s| {
  s.spawn(|| {
    GLOBALS.set(&globals, || {
      try_with_handler(cm.clone(), Default::default(), |handler| {
        HELPERS.set(&Helpers::new(false), || {
          HANDLER.set(handler, || {
            transform_by_swc_plugins(/* ... */)
          })
        })
      })
    })
  })
  .join()
  .map_err(|e| anyhow!("swc wasm plugin worker panicked: {e:?}"))?
});
```

Notes:

- `std::thread::scope` borrows references, so we avoid the `'static`
  bound that `std::thread::spawn` would impose.
- All scoped-tls slots (`GLOBALS` / `HANDLER` / `HELPERS` / `COMMENTS`)
  must be re-set inside the new thread; otherwise SWC passes panic
  because TLS is not inherited across OS threads.
- Because the work runs on an OS thread rather than a rayon worker, the
  cranelift/rayon nested scheduling can no longer starve it.

### 1.4 Removing `PLUGIN_TRANSFORM_LOCK`

Before introducing `std::thread::scope`, we tried "serialize wasm
transform with a global Mutex". After the OS-thread isolation landed:

- The rayon nested starvation is already gone;
- wasmer's internal lock is sufficient to protect the runtime;
- A global lock would serialize transforms across all modules and
  significantly hurt build throughput.

Conclusion: `PLUGIN_TRANSFORM_LOCK` and `run_serialized_plugin_transform`
were removed (we kept `PLUGIN_TOKIO_RT`, `PLUGIN_MODULE_CACHE`, and
`block_on_plugin_transform`). The three `block_on` unit tests in
`crates/plugin_script/src/swc_plugins.rs` still pass.

### 1.5 Verification commands

```bash
FARM_THREAD_NUMS=2 node scripts/test-examples.mjs --skip-build-js-plugins --example emotion
node scripts/test-examples.mjs --skip-build-js-plugins --example emotion
pnpm run test-e2e -- --grep emotion
```

## 2. svgr-rs cargo test Fix Playbook

### 2.1 Symptoms

`cargo test -p farmfe_svgr_rs` reported:

- `unresolved module or unlinked crate `swc_core``
- `unresolved module or unlinked crate `testing``
- doctest `unresolved import `svgr_rs``

### 2.2 Steps

1. **Replace `swc_core` re-exports** with the underlying crates, e.g.
   - `swc_core::common::*` → `swc_common`
   - `swc_core::ecma::codegen::*` → `swc_ecma_codegen`
2. **Drop the `testing` dependency**: remove `#[testing::fixture]` and
   `testing::NormalizedOutput`; iterate `__fixture__/<name>/input.svg`
   manually with `std::fs::read_dir` and compare against `output.jsx`:
   ```rust
   for entry in fs::read_dir(&fixtures_dir)? { /* document_test(input) */ }
   assert_eq!(
     res.trim_end_matches(['\n','\r']),
     expected.trim_end_matches(['\n','\r']),
   );
   ```
3. **Emitter API change**: `emit_module_item` is gone, use the `Node`
   trait instead:
   ```rust
   use swc_ecma_codegen::Node;
   item.emit_with(&mut emitter)?;
   ```
4. **doctest crate name**: the crate name in `Cargo.toml` is
   `farmfe_svgr_rs`, so the doctest in `lib.rs` must use
   `use farmfe_svgr_rs::transform;` rather than `svgr_rs`.

### 2.3 Real typo fix

`src/transform_svg_component/variables.rs` had 5 occurrences of
`soruce_value` (2 in fn signatures + 3 at call sites). These are real
typos and should be fixed in code — **do not** add them to the cspell
dictionary.

### 2.4 Verification

```bash
cargo test -p farmfe_svgr_rs
# 66 unit tests + 1 doctest pass
```

## 3. cspell Fix Strategy

### 3.1 Two categories of errors

| Category | Action |
| --- | --- |
| Real typo (e.g. `soruce`, `recieve`) | Fix the source; **do not** add to dictionary |
| Legitimate identifier (SVG/HTML attrs, API names, wasm jargon, ...) | Add to [cspell.json](file:///Users/bytedance/Desktop/opensource/farm-v2/cspell.json) `words` array |

### 3.2 Operational tips

- Run `pnpm cspell "**" --gitignore --no-progress` to list every issue,
  then group them by file and triage.
- Keep `cspell.json` valid JSON when editing — losing keys like
  `"ignorePaths":` will turn every cspell run into a parse error.
- Append words alphabetically for easier review. This pass added 200+
  SVG/HTML attribute identifiers in one go (`viewbox`, `xlinkhref`,
  `tspan`, `SVGSVG`, `dasharray`, ...).

### 3.3 Acceptance

```bash
pnpm cspell "**" --gitignore --no-progress
# Files checked: 1512, Issues found: 0 in 0 files.
```

## 4. General Lessons

1. When something hangs, **collect stacks first** (macOS `sample`, Linux
   `gdb`/`perf`) — don't guess.
2. Inside a `rayon` pool, be very careful with FFI calls (wasm/native)
   that **internally spawn rayon work** or **hold a global lock for a
   long time**; they can cause nested starvation. Prefer
   `std::thread::spawn` or `std::thread::scope` to isolate them.
3. When crossing into an OS thread, remember to re-enter scoped-tls
   slots (`GLOBALS` / `HANDLER` / `HELPERS` / `COMMENTS`, tokio runtime
   handles, etc.) — they are not inherited automatically.
4. "Add a lock and the deadlock goes away" is often a symptom-only fix.
   If the hang persists with the lock in place, the root cause is
   usually **scheduler starvation** rather than a data race; switch the
   approach to "move the suspicious work off the current scheduler".
5. When a test infra dependency is unstable (e.g. `testing` /
   `swc_core` shifts), falling back to `std::fs` + a hand-written walker
   is the most reliable option.
6. For cspell failures, **classify before bulk-editing**: keep real
   typos and legitimate identifiers in separate buckets.
