# Commerce functional acceptance

The Commerce Lab is an existing deterministic merchant application, imported
from `2574b9c1931f01849a2b4152fb5bc39a905496fc`. It uses `createFixtures` and
`createMockApi`; acceptance must not connect real payment or order services.
Historical reports mentioned in its README are not evidence for a new candidate.

## Source-only checks

After installing the workspace dependencies without build scripts, run:

```sh
cd examples/commerce-lab
npm run test:unit
node benchmark/extension-generator.mjs baseline
cd ../..
node --test examples/ssr-utils/render-dev.test.mjs
pnpm exec vitest run packages/core/tests/library-output-config.spec.ts --maxWorkers 1 --minWorkers 1
```

The unit command does not load native plugins or require emitted workspace
packages. `npm test` retains the complete suite, including native configuration
purity and emitted-artifact provenance checks; run it after building prerequisites.
The generator refuses to overwrite modified fixtures. Generated extensions and
reports are ignored by Git. The catalog test transforms its generated TypeScript
before executing it, so it does not depend on Node's experimental TypeScript loader.

## Native prerequisites and provenance

Build or verify source-matched Core, CLI, utils, runtime, HMR and import-meta
packages. JavaScript pipelines also need emitted Sass, SVGR and Tailwind packages.
Core needs the replace-dirname native plugin. All pipelines use React and Worker;
the complete Rust pipeline additionally exercises auto-import, SVGR, Sass,
Tailwind, YAML, DSV, MDX, compress and WASM. The native adapters also exercise
virtual modules, URL, image, icons, strip, modular-import and react-components.
SDK and Vue configurations require DTS, Vue and Vue JSX plugins respectively.

Record native paths and SHA-256 with `captureNativeProvenance`; record emitted
workspace and lockfile identity with `captureSourceProvenance`. Do not satisfy
these checks with placeholder dist files or substitute an unrelated old binary.
Vue SFC/JSX changes require freshly built matching plugins. Unchanged compiler
inputs can reuse verified compatible v2 artifacts only when their provenance is
known. Package version equality alone does not establish compatibility.

## Sequential functional matrix

Only start builds in the coordinated native window. Maintain the local disk
guard (8 GiB) and physical free-memory guard (3.4 GiB); do not stop unknown
processes or modify another task's target, preview or evidence. Performance
sampling can be deferred, while functional checks remain required.

For each of `js`, `rust` and `mixed`, set `COMMERCE_GROUP`, use an owned cache ID,
then run build, start and preview. From the example directory, the existing
`tests/browser.mjs` checks a running development server; `tests/orders-browser.mjs`
builds and previews selected groups sequentially and writes its own reports.
Use `COMMERCE_BROWSER_GROUPS` for an explicit subset. Its preview uses port 4352;
check ownership before starting it. Do not run two drivers on that port.

| Area | Required assertions |
| --- | --- |
| Tenants and roles | merchant isolation; viewer mutation refusal; forged tenant rejection |
| Products and imports | edits, atomic publication, CSV/Worker validation and failure recovery |
| Orders and inventory | virtualized list bounds, filtering, refund accounting, safe receipts and persistence |
| Campaigns and channels | policy validation, pricing, publication and merchant isolation |
| Assets and data | SVG/images, MDX help, YAML/DSV, WASM, Worker, gzip integrity |
| Charts and runtime | rendered SVG charts, correct totals, no unexpected pageerror, console error or failed request |
| Vue and SDK | dedicated SFC/JSX/HMR/type drivers, consumer output and type updates |

Default Commerce configs explicitly disable lazy compilation. Full/lazy claims
require additional reviewed configuration overrides and the same browser
assertions; merely labeling a report as lazy does not enable it. Passing unit
tests or a minimal page smoke does not satisfy this matrix.

## Library configuration contract

`compilation.output.libraryBundleType` now accepts Rust's existing values:
`single-bundle`, `multiple-bundle` and `bundle-less`. It applies to
`targetEnv: 'library'`. Omitting it leaves the native `single-bundle` default
unchanged. Single-bundle currently requires one entry; multiple-bundle uses the
existing partial-bundling/code-splitting logic; bundle-less sets the existing
per-module bundling limits. This exposes existing behavior without changing
its native default or promising support beyond the Rust implementation.
