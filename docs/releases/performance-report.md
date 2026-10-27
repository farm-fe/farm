# Commerce CI performance report

The optional three-OS job follows Examples acceptance and does not rebuild Rust.
Every native download names the current workflow SHA and exact platform/plugin.
The first protocol uses Commerce's deterministic baseline extension generator
(four categories), Rust configuration, and existing quantile/CSV implementation.
It copies the workload into an owned temporary directory and links read-only
consumer dependencies. Tracked Commerce sources are fingerprinted before/after;
native hashes are recorded and checked for drift. Cache and output paths are
inside that temporary workload and are removed after sampling.

One cold warmup is unscored, followed by three serial cold/warm pairs. Each
cold pair starts with a distinct cache; warm reuses that successful cold cache.
Without independent cache-hit evidence this means **reusable**, not confirmed
cache hit. JS/CSS sizes are summed from actual uncompressed output and Node gzip;
source maps and precompressed sidecars are excluded. Failed samples remain
failed/null and are never replaced with zero. Reports always say
`publishable:false`: three pairs are exploratory, not performance certification.

Artifacts contain raw JSON, summary JSON/Markdown, CSV and build logs. A Step
Summary includes median/IQR and errors. Each command has a two-minute timeout
with owned process-tree termination; the sampler has an eighteen-minute budget
and the job a twenty-minute limit. Setup failures are visible as failed steps;
they do not count as successful or zero-cost samples.

## Comparison boundaries and next stages

This implementation is **current-only** until an exact-base native cohort plus
compatible workload/toolchain receipt is available. Merely having a historical
summary is insufficient for a same-runner paired result. Compatibility dimensions
are protocol, fixture hash, Node, pnpm, OS, architecture and dependency lock hash.
No regression percentage or cross-OS absolute ranking is emitted.

The next stage should materialize base/head consumers on the same runner, verify
both cohorts against their producer SHAs, and alternate paired samples. Historical
trends are separate from those paired results. A sticky PR summary can consume
the uploaded JSON in a separate trusted workflow; do not execute PR code via
`pull_request_target` or introduce broad write permissions into this sampler.

There is no verified Vite Commerce build yet: using Vite-compatible plugins inside
Farm is not a Vite oracle. A fair comparison needs equivalent routes/assets/plugin
semantics and pinned Vite dependencies before a second bundler can be reported.
HMR needs source-edit isolation and restoration proof. RSS/CPU needs equivalent
process-tree accounting on all three systems; the existing Windows-only sampler
must not be presented as a cross-platform measurement.
