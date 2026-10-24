# Maintained plugin diagrams

The `.mmd` files are the editable source. Paired light/dark SVGs are generated
assets, not a replacement for the text descriptions in the API pages. The manifest
binds each source and SVG by SHA256. API names and cache/update branches must be
checked against the current Rust trait, plugin driver and compiler call sites.

Normal `farm build`, CI verification and website readers do **not** install or
execute Mermaid. This optional authoring step uses Node.js and Chromium; it is not
part of the Rust compilation claim. No Mermaid runtime enters page bundles.

## Regenerate

Use an isolated authoring directory with Node >=22.13 and a lockfile. Install
`@mermaid-js/mermaid-cli@12.0.0`, `mermaid@12.2.0` and
`playwright-core@1.55.0`; set `PUPPETEER_SKIP_DOWNLOAD=true` to avoid a second
browser download. The script uses the official Mermaid browser bundle through
Playwright, not the CLI's Puppeteer launcher. Provide an existing Chromium and the
installed Playwright module through `FARM_DIAGRAM_CHROMIUM` and
`FARM_DIAGRAM_PLAYWRIGHT`.

Run `node scripts/generate-doc-diagrams.mjs <authoring-directory>/node_modules/@mermaid-js/mermaid-cli/src/cli.js`.
Then run `node --test scripts/check-doc-diagrams.test.mjs`.

Do not hand-edit SVGs. Strict Mermaid mode and native SVG labels disable scripts,
interactive links and HTML labels. The checker rejects stale sources, changed
outputs, external resources, scripts, event attributes and `foreignObject`.
Generated SVGs contain no embedded fonts. Pages retain captions, API prose and
links to the public Mermaid sources for agents and accessible text readers.

## Farm brand presentation

The light and dark diagrams use the website's purple brand and neutral surfaces.
Stage panels separate initial compilation, per-module/cache processing, resource
generation and independent updates. Purple outlines indicate first/bail callbacks;
neutral outlines indicate serial callbacks; muted teal outlines indicate parallel
callbacks. These colors describe hook dispatch, not global compiler concurrency.
The four Rust lifecycle sources include all 35 current hooks. Their original
conditions, cache branches and observation seal remain explicit. The generator
adds presentation classes; do not replace a conditional branch with a shorter
linear path just to fit an image.

The source includes conditional module/resource caches and a separate update
entrypoint; `finish` does not automatically call `update_modules`. A runtime
`first` hook stops on a truthy value, whereas Rust `first` dispatch uses an
optional handled result. A serial callback does not make the entire compiler
single-threaded.
