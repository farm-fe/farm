# Real Vite comparison: Commerce catalog/quote subset

Protocol `commerce-catalog-quote-subset-v1` compares Farm's native compiler with
the actual Vite JavaScript API, pinned to **Vite 5.2.8**. The adapter checks the
resolved package version before any build or Vite server startup and fails on a
mismatch. This is a separate workload cohort from complex Commerce Lab; never
compute a ratio between these subset results and full Commerce Rust/JS results.

Both builders consume the same generated directory: 240 deterministic products,
category filtering, text search, a dynamic-import product detail/quote panel,
CSS, and an exact copy of Commerce Lab's existing `business-quote.mjs`. It covers
integer-cent pricing, tier discounts, shipping and reserved-stock rejection.
There is no Farm Vite-plugin adapter in this comparator. It does **not** claim
equivalence for the full React/Vue apps, Rust plugins, MDX, SVG, Sass, Tailwind,
workers, Wasm, DSV/YAML, component libraries or the full generated extension graph.

`examples/commerce-lab/benchmark/vite-comparison.mjs` exports:

- `prepareComparisonFixture(root)`: write identical deterministic sources and
  return protocol, source SHA-256 fingerprint, file list and product count.
- `comparisonConfigs(root, { port })`: matched ES2017, minified, no sourcemaps
  production configs, with Farm persistent cache and lazy compilation disabled.
- `assertViteVersion()`: return checked package path/version.
- `runComparisonBuild('farm' | 'vite', root)`: import the real builder, time only
  `build(config)`, then record every emitted file's raw bytes, gzip level-9 bytes,
  and SHA-256. Imports and output collection are outside the build timer.
- `startViteComparison(root, { port })`: real `vite.createServer`/`listen`, HTTP
  readiness, URL and `close()`; the returned server is owned by the caller.
- `verifyComparisonHttp(url)`: check actual HTML and every entry-script response;
  this deliberately reports `functional: false` and is only a readiness check.
- `verifyComparisonBrowser(page, url)`: Playwright-compatible functional acceptance
  for both dev and built previews. Verify 240 products, 80 in one category, one
  search match, lazy detail rendering, totals 2400/3200 cents and stock rejection,
  with no uncaught page errors. A successful browser result is required before
  calling the corresponding run functionally equivalent.

The main runner must use isolated child processes for each tool/sample, record
the same fingerprint on both sides, start Farm through its own native API/CLI,
and serve each production directory for browser acceptance. Stop/close every
server after inspection. Run serially on the same host and report Node/tool
versions, samples and cache policy. Startup timings from different timer
boundaries must not be compared; `startViteComparison.readyMs` starts immediately
before listen and includes HTTP readiness, not process spawn or module import.
Gzip sizes are computed for every emitted file, without HTTP headers; they are
comparable size evidence rather than precompressed artifact delivery.

Lightweight tests: `node --test examples/commerce-lab/tests/vite-comparison.test.mjs`.
These test deterministic sources, actual installed version, business outcomes,
matched config and size accounting. They are not browser/build acceptance.

## Local acceptance, 2026-10-27

Actual Farm 2.0.0 and Vite 5.2.8 builds ran serially on Node 22.4.0, using existing
native binaries. Source fingerprint:
`f1260573f20a117eb462eea79ff552695d777d1844ed3abf168fc7b3e49b4b27`.
Playwright Chromium 1.55.0 passed all seven functional checks for **both dev
servers and both production previews**, with no page, console or request errors.
Every owned browser and server was closed afterward.

| Tool | API build, ms | Emitted raw bytes | Gzip level-9 bytes |
| --- | ---: | ---: | ---: |
| Farm 2.0.0 | 70.5388 | 31,143 | 7,752 |
| Vite 5.2.8 | 432.0356 | 25,126 | 5,392 |

These are one diagnostic sample per tool, not a performance distribution or a
full Commerce Lab claim. Free RAM before each build was 3.5838/3.5338 GiB; free
disk was 25.1937 GiB, above the required 3.4/8 GiB guards. The execution workspace
preserves `vite-comparison-evidence/report.json` alongside the checkout, including
per-file hashes, exact resolved Vite package path and browser acceptance records.
