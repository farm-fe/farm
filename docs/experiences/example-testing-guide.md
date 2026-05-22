# Example Testing Guide (`scripts/test-examples.mjs`)

## Scope

This guide explains how to validate one example or a subset of examples using
`scripts/test-examples.mjs`, including the argument mapping and expected output.

## Prerequisites

- Run from repository root.
- Dependencies are installed (`pnpm install`).
- Use `--skip-build-js-plugins` when iterating on non-JS-plugin changes to
  reduce validation time.

## Quick Start

Build only one example:

```bash
node scripts/test-examples.mjs --skip-build-js-plugins --example rust-plugin-wasm
```

Build from one example to the end of the sorted examples list:

```bash
node scripts/test-examples.mjs --skip-build-js-plugins --from rust-plugin-wasm
```

## Argument Mapping

| Argument | Meaning | Notes |
| --- | --- | --- |
| `--example <name>` | Build only one example. | Supports `--example=name` and `FARM_EXAMPLE`. |
| `--from <name>` | Build from the given example to the end. | Alias: `--start-from`. Supports `--from=name` / `--start-from=name` and `FARM_EXAMPLE_START_FROM`. |
| `--skip-build-js-plugins` | Skip JS plugin build stage. | Alias: `--skip-build-js-plugin`; also supports `--skip-build-js-plugins=true` and `FARM_SKIP_BUILD_JS_PLUGINS=1/true`. |

Precedence in example selection:

1. `--example` (single example mode)
2. `--from` / `--start-from` (range mode)
3. Default: build all examples

## Expected Output Signals

For single-example mode:

- `Building only example: <name>`
- `Building 1 examples...`
- `Building examples\\<name>`

If the example does not exist:

- `Example '<name>' was not found under ./examples`

## Troubleshooting

- If you see another example (for example `examples\\arcgis`) while using
  `--example`, verify that:
  - You are running the command from repo root.
  - The command uses `--example`, not only `--from`.
  - `<name>` exactly matches a directory under `examples/`.
- If command exits with non-zero, inspect the first failed example build log to
  determine whether it is an example issue or orchestration issue.

## E2E Preview Freshness

`scripts/test-e2e.mjs` can be run standalone, without a preceding
`scripts/test-examples.mjs` build. For `startAndTest(..., 'preview')`, the runner
only runs `npm run build` first when the example's Farm config enables
`server.writeToDisk: true`. Those examples write production-like files during dev
server runs, so preview needs a fresh production build to avoid serving stale dev
artifacts. Other examples skip this pre-preview build to keep E2E fast and avoid
duplicating the example build pipeline.

Pre-preview builds use the example's normal persistent-cache setting so E2E can
cover cached production builds. Do not serialize these builds in the E2E runner:
independent worker processes should be able to build concurrently, and that
concurrency is part of the cache coverage this suite is meant to exercise.

The E2E orchestrator also scans for stale Farm E2E processes before and after
each run on Linux, macOS, and Windows. Processes older than five minutes that
belong to a previous `test-e2e` tree are terminated, including orphaned worker,
dev-server, build, and Playwright Chromium children. Unix platforms clean process
groups; Windows cleans process trees. Stale orphaned Playwright Chromium
processes are also cleaned when they no longer have an owning parent. This keeps
each run independent without killing the current run. The stale threshold
defaults to five minutes and can be overridden with
`FARM_E2E_STALE_PROCESS_SECONDS` for local diagnostics.

This avoids a subtle stale-artifact failure mode: `farm start` serves current
source, while `farm preview` serves the existing production output directory.
If that directory was produced by an older source/config/cache state, preview
can fail with misleading browser runtime errors even though dev-server mode
passes. Before debugging module concatenation, tree shaking, or minification,
compare normal cached builds with `FARM_DISABLE_CACHE=1` builds. If disabling
persistent cache fixes errors such as `e.replace is not a function`, inspect the
module cache restore path before changing example source. In particular, cached
script modules must restore SWC marks to the matching fields: `top_level_mark`
from the resolver's `top_level_mark`, and `unresolved_mark` from
`unresolved_mark`. Swapping them can make cached builds generate runtime export
or helper mismatches that look like minify or module-concatenation bugs. Keep the
normal cached, concurrent preview builds enabled in E2E so this class of cache
regression remains observable.
