# Commerce CI performance report

The optional three-OS job follows Examples, Docs and consumer acceptance, uses
its own exact-SHA native artifacts, and never rebuilds Rust. macOS remains the
standard ARM64 cohort. No local measurement is implied by the implementation.

## Complete workload and timing

Protocol `commerce-business-2048-supervised-v1` generates **business-2048** with
`business: true`: 2,048 categories, 14 unique files per category plus index,
28,673 generated source files. Base input counts are measured separately; files
are never relabeled as module counts. Original HTML, admin entry, lazy pages,
adapters, data, worker and WASM inputs stay intact. The temporary project also
receives its inherited root TypeScript configuration. The complete copied input
closure, generator, lockfile and supervision harness are fingerprinted.

A complete cold/warm bootstrap is not scored. Three scored pairs follow,
alternating tool/cohort order serially on the same runner. Each sample uses a new
process, and both tools report full child-process wall clock including config,
compiler startup and gzip output generation. Fixture generation, extra graph
compilation, browser checks and normalized JS/CSS size collection are outside
the measured child. Gzip sidecars use level 6 and threshold 1,024 without deleting
originals; normalized size accounting uses Node gzip level 9 on raw JS/CSS only.

Farm warm samples reuse the successful cold cache directory, without asserting
an independently proven cache hit. Vite 5.2.8 production Rollup cache is disabled;
its second build is **repeat**, not persistent-cache reuse. Repeat samples remain
in raw JSON/CSV and never generate a warm-cache speed ratio.

Failed, interrupted and absent samples remain null. Timeout, incomplete pairs,
missing source coverage or browser failure makes the report invalid. No workload
reduction or zero substitution is allowed. Three pairs are exploratory and all
reports retain `publishable:false`; these are not release performance thresholds.

## Same-run resource supervision

Identity and raw resource observations are written before admission. Darwin free
pages remain free-page observations, never invented available physical memory.
Python 3.12 standard-library supervision queries kernel memory pressure on macOS,
PSI plus MemAvailable on Linux, and low-memory notification plus available bytes
on Windows. Unknown/failed probes and warning/critical pressure stop the owned
build. Swap/pageout observations retain their source and units; unsupported
Windows paging counters remain explicitly unavailable.

The existing 8 GiB emergency disk reserve is retained. Initial supervised cold
and graph command caps are 240 seconds, warm 180 seconds, bounded by the same
18-minute total sampling deadline and 20-minute job limit. This replaces the
previous universal 120-second cap, which timed out the Windows complete workload
without proving whether compilation was merely slow. After full successful
bootstrap pairs and confirmed cleanup, this run records observed durations, disk
growth and RSS. Later admission conservatively requires twice the larger observed
root/group RSS where actual available bytes exist, plus observed disk growth over
the reserve. Caps derive from bootstrap time with margin and never exceed their
initial cap or remaining deadline. This is same-run evidence, not a portable
capacity certification or permission to reduce safety floors elsewhere.

Unix builds own a process group. Windows launches suspended into a kill-on-close
Job Object before resuming. Every exit/timeout verifies group/job cleanup; unknown
cleanup stops further sampling and preserves temporary inputs. Kernel root peak
RSS and sampled summed-group RSS are reported separately, not called footprint
or commit. Group RSS may double-count shared pages; live CPU totals are diagnostic,
not formal CPU benchmarks. Phase, cwd, allocated timeout, elapsed time, pressure,
disk, RSS, cleanup and bounded sanitized log tails are retained. No global process
kill is used. Actual OS behavior must be verified by the new remote CI run.

## Base, Vite and history

Base/head require identical fixture, harness, Node/pnpm/Python, OS/architecture,
runner image/version and lockfile. Exact-base native artifacts are independently
validated; head native artifacts are never substituted. Base JS preparation uses
the same supervisor. Incompatible or missing base becomes explicit current-only.
An unconfirmed base process tree prevents head sampling.

The main Vite comparison now uses `commerce-full-business-2048-v1`, the same
original app and generated inputs, ESNext, sourcemaps and minification. Thin hooks
reuse existing content/AST implementations for virtual modules, media, component
imports, strip, data and WASM. Existing locked SVGR, Tailwind Vite and Markdown
plugins are resolved from explicit owning workspace manifests, not assumed hoists.
The existing pipeline badge labels Vite as Vite, rather than claiming Rust plugins.
GFM tables are explicitly enabled in both MDX implementations. The old catalog/
quote subset remains a separate historical microbenchmark and is not sampled by
this main protocol or mixed into its history.

Unscored Farm and Vite graph checks prove coverage of all generated source files,
classify CSS/assets/query/virtual identities and keep emitted module counts
separate. Vite worker graphs are listed separately; Farm's parent trace does not
claim to expose the nested native worker compiler graph. Both built outputs run
the same browser assertions, including first/middle/last category interaction,
stock/discount/shipping rules, MDX/SVG/styles, original worker import and WASM type
generation. Three category interactions do not claim all-category UI coverage.
Page/console/request/HTTP errors invalidate the result. Only local mock data is used.

History reads at most ten prior trusted runs and distinguishes missing, expired,
incompatible and invalid evidence. Artifacts retain 14 days; history is not permanent
storage. Protocol/harness/fixture/toolchain/image cohorts must match. No cross-OS
absolute ranking is made. Complete full-tool sampling may exhaust the current job
budget; that remains invalid evidence, not a reduced fixture or fewer valid pairs.

## Reports and trust

Artifacts include raw/summary JSON, CSV, logs, resource traces, graph detail and
browser evidence. All three platforms remain visible when reports are absent or
invalid. A trusted default-branch reporter validates run, repository, attempt,
current PR/head/base and bounded summary identities before updating one bot comment;
old SHAs/attempts cannot overwrite newer evidence. No artifact code is executed,
no new token is created, and `pull_request_target` is not used. The reporter must
exist on the default branch before live comments work; source implementation is
not proof that a comment was posted. HMR benchmarking remains outside this protocol.
