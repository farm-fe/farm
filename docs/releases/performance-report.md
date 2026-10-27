# Commerce CI performance report

The optional three-OS job follows Examples acceptance and does not rebuild Rust.
Every native download names the current workflow SHA and exact platform/plugin.
The first protocol uses Commerce's deterministic baseline extension generator
(four categories), Rust configuration, and existing quantile/CSV implementation.
It copies the workload into an owned temporary directory and links read-only
consumer dependencies. Tracked Commerce sources are fingerprinted before/after;
native hashes are recorded and checked for drift. Cache and output paths are
inside that temporary workload and are removed after sampling.

One cold warmup is not scored, followed by three serial cold/warm pairs. Each
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

## Same-runner comparison and history

The job checks out the exact base separately, discovers its successful trusted
native cohort and verifies any synthetic producer commit by full source-tree
equality. It prepares only JavaScript consumers, never compiling base Rust or
using head binaries for base. It alternates base/head samples on the same runner
when protocol, actual workload fingerprint, Node, pnpm, OS, architecture and lock
hash agree. Missing or incompatible base evidence produces **current-only** with
an explicit reason, not a regression conclusion.

Comparable history is read from up to ten successful prior GitHub artifact runs.
OS, architecture, runner image/version, protocol, fixture, lock and toolchain
must agree. Missing, expired (14-day retention), incompatible and invalid entries
are distinguished. History is separate from same-runner paired measurements.
No Pages, external service, cross-OS absolute ranking or release threshold is used.

The separate `commerce-catalog-quote-subset-v1` fixture actually builds with Farm
and pinned Vite 5.2.8: identical catalog/search/category/lazy-detail/quote source,
ES2017 output and minification, with persistent caching disabled for both. One
warmup plus three alternating scored API builds report median/IQR and emitted
raw/gzip bytes. This is a common subset, not full Rust Commerce parity. It is
shown in a separate table and never compared with the full Rust workload timings.
After the timed samples, each OS opens both built outputs in Chromium and checks
catalog, category, search, lazy detail, quote, quantity and stock rejection.
Page/console errors and failed requests invalidate the report. Browser startup
and correctness checks are outside the timing samples; local mock tests alone
do not establish that these three-OS browser checks have passed.
See [the Vite comparison boundary](./vite-comparison.md).

Three platform reports are compacted into a strict bounded JSON envelope. A
separate default-branch `workflow_run` reporter validates API-confirmed run,
repository, current PR/head/base and artifact identities before updating one bot
comment. Old SHAs and older run IDs/attempts cannot overwrite a newer result.
The same run's newer attempt can replace an earlier partial result. Artifact
lookup is bounded and paginated beyond the native artifact collection. All three
OS entries remain visible even when a platform report is missing or invalid;
base/history exclusion reasons are retained rather than silently discarded.
Workload fingerprints include HTML, adapters, scale profiles and business source.
An independent three-OS Node test job exercises these reporting contracts.
It checks
out only trusted reporter source, never PR code, and does not use
`pull_request_target`. The reporter must first exist on the default branch;
local mock tests do not establish that a live PR comment was posted.

HMR needs source-edit isolation and restoration proof. RSS/CPU needs equivalent
process-tree accounting on all three systems; the existing Windows-only sampler
must not be presented as a cross-platform measurement.
