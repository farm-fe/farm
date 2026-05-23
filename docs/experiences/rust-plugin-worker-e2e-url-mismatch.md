# Rust Plugin Worker E2E URL Mismatch Troubleshooting

## Scope

This document records the debugging experience for `examples/rust-plugin-worker`
E2E failures where worker cases fail in the browser even though the example
build itself succeeds.

The failure was observed on Windows and macOS CI in `pnpm run test-e2e`.

## Typical Symptoms

The E2E runner reports a browser console error from the default smoke test:

```text
[rust-plugin-worker] rust-plugin-worker › start
  Browser console error: Worker case failures [Object, Object, Object, Object, Object, Object, Object]
```

Important signals:

- The Farm build or dev server starts successfully.
- The failure comes from the page's own `console.error("Worker case failures", results)`.
- Browser logs may only show `[Object]`, so the failing worker case names are
  hidden unless the page or bundle is inspected more directly.
- The issue can appear platform-dependent because worker dynamic-entry names
  can be selected differently when multiple worker syntaxes reference the same
  source file.

## Investigation Checklist

### 1. Inspect the worker case source first

`examples/rust-plugin-worker/src/index.tsx` runs several worker styles against
the same `vue.worker.ts` source:

- `new Worker(new URL("/src/worker/vue.worker.ts", import.meta.url))`
- `new Worker(new URL("./worker/vue.worker.ts", import.meta.url))`
- `import WorkerCtor from "./worker/vue.worker.ts?worker"`
- `import workerUrl from "./worker/vue.worker.ts?worker&url"`

If the same source file produces multiple public worker URLs, only one may be
emitted because Farm tracks dynamic entries by the real source `ModuleId`.

### 2. Build the example and compare wrapper URLs with emitted files

Run:

```bash
cd examples/rust-plugin-worker
npm run build
```

Then inspect `dist/index.*.js` and the emitted worker files in `dist/`.

A broken build can look like this:

```js
function WorkerWrapper(options) {
  return new Worker("/vue.63ddc497.worker.js", options);
}
var vue_worker_ts_default = "/vue.c1e83ed7.worker.js";
```

while the output directory only contains:

```text
dist/vue.63ddc497.worker.js
```

That means `?worker&url` points at a non-emitted file.

### 3. Do not trust preview `200` alone

Farm preview may return `index.html` fallback for a missing worker URL. A request
can therefore return status `200` while still being the wrong content.

Check the response body, not only `response.ok`:

```powershell
$r = Invoke-WebRequest -Uri http://localhost:1911/vue.c1e83ed7.worker.js -UseBasicParsing
$r.Content.Substring(0, [Math]::Min(120, $r.Content.Length))
```

If the response starts with `<!doctype html>`, the worker URL is missing and the
browser is loading HTML as a worker script.

### 4. Rebuild the Rust plugin before validating examples

When changing `rust-plugins/worker/src/*.rs`, rebuild the native plugin before
running the example. Otherwise `farm build` can still load the old local
`index.farm` binary.

```bash
pnpm --filter @farmfe/plugin-worker build
pnpm run test-e2e -- --example rust-plugin-worker
```

## Root Cause

The worker plugin computed dynamic-entry names from the worker source filename
and used the import `module_id` bytes as the hash input.

That made the entry name depend on query parameters:

```text
src/worker/vue.worker.ts?worker      -> vue.63ddc497.worker.js
src/worker/vue.worker.ts?worker&url  -> vue.c1e83ed7.worker.js
```

But Farm stores dynamic entries by the resolved source module. These imports all
resolve to the same `vue.worker.ts` source, so the module graph can only keep one
dynamic-entry name for that source.

The mismatch is:

- Wrapper modules are generated immediately and may point at different URLs.
- The module graph emits only one worker resource for the resolved source.
- Any wrapper pointing at the non-selected URL fails at runtime.

There was a second platform-sensitive detail: resolved paths can contain `\` on
Windows and `/` elsewhere, or differ by resolve branch. If the path string is
hashed without normalization, equivalent source files can still produce different
entry names across platforms or call sites.

## Effective Fix

Make worker entry names stable for the real worker source:

- Use the resolved worker source path as the hash input instead of the virtual
  `module_id` with query parameters.
- Normalize path separators before hashing (`\` -> `/`).
- Keep the worker filename stem from the normalized source path.

The regression coverage should include both cases:

- Same source with different queries: `?worker` and `?worker&url`.
- Same source with different path separators: `C:\project\...` and
  `C:/project/...`.

## Validation

Minimum validation after the fix:

```bash
cargo test -p farmfe_plugin_worker worker_entry_name_is_stable
pnpm --filter @farmfe/plugin-worker build
pnpm run test-e2e -- --example rust-plugin-worker
```

Expected result:

```text
cargo test ... 2 passed
rust-plugin-worker › start    passed
rust-plugin-worker › preview  passed
```

Also inspect the rebuilt `dist/index.*.js`: all references to the same
`vue.worker.ts` source should point at the same emitted `vue.*.worker.js` file.

## Takeaways

- When a plugin creates URL wrappers before the compiler emits resources, the
  wrapper's predicted URL must use exactly the same identity as the compiler's
  dynamic-entry emission path.
- Do not include query parameters in the identity for a worker source if the
  module graph deduplicates by resolved source module.
- E2E checks that only assert `fetch(url).ok` can miss missing-asset bugs when a
  preview server falls back to HTML. Validate response content for generated
  script URLs when debugging worker failures.
- For Rust plugin changes, always rebuild the platform plugin binary before
  validating examples.