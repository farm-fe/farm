<!-- cspell:words pageout microbenchmark errno -->
# Commerce CI performance report

The non-blocking PR job for three OS platforms follows Examples and Docs acceptance, uses
its own exact-SHA native artifacts, and never rebuilds Rust. macOS remains the
standard ARM64 cohort. No local measurement is implied by the implementation.

## Complete workload and timing

Protocol `commerce-business-2048-farm-supervised-v1` generates **business-2048** with
`business: true`: 2,048 categories, 14 unique files per category plus index,
28,673 generated source files. Base input counts are measured separately; files
are never relabeled as module counts. Original HTML, admin entry, lazy pages,
adapters, data, worker and WASM inputs stay intact. The temporary project also
receives its inherited root TypeScript configuration. The complete copied input
closure, generator, lockfile and supervision harness are fingerprinted.
The inherited root TypeScript config and explicit dependency-owner manifests are
also hashed. Isolated projects retain minimal owner manifests/node_modules links
for Markdown resolution; these are not extra application inputs.

A complete cold/warm bootstrap is not scored. Three scored pairs follow,
alternating head/base cohort order serially on the same runner. Each sample uses a new
process, and Farm reports full child-process wall clock including config,
compiler startup and gzip output generation. Fixture generation, extra graph
compilation, browser checks and normalized JS/CSS size collection are outside
the measured child. Gzip sidecars use level 6 and threshold 1,024 without deleting
originals; normalized size accounting uses Node gzip level 9 on raw JS/CSS only.

Farm warm samples reuse the successful cold cache directory, without asserting
an independently proven cache hit. CI measures Farm only: no Vite dependency
preparation, warmup, measured build, graph compilation or browser run is scheduled.
Every enabled Farm cohort must have exactly three valid cold and three valid warm
samples; missing samples cannot pass through an empty-array success check.

Failed, interrupted and absent samples remain null. Timeout, incomplete pairs,
missing source coverage or browser failure makes the report invalid. No workload
reduction or zero substitution is allowed. Three pairs are exploratory and all
reports retain `publishable:false`; these are not release speed thresholds. The
stable release gate nevertheless requires complete valid reports on all three
platforms, independently of continue-on-error workflow behavior.

## Same-run resource supervision

Identity and raw resource observations are written before admission. Darwin free
pages remain free-page observations, never invented available physical memory.
Python 3.12 standard-library supervision queries kernel memory pressure on macOS,
PSI plus MemAvailable on Linux, and low-memory notification plus available bytes
on Windows. Unknown/failed probes and established warning/critical pressure stop
the owned build. Linux PSI is system-wide contention, not proof that this build
caused OOM. Raw kernel `avg10` and total stall counters remain recorded. One tiny
counter increment does not stop the run: the supervisor uses a five-second window,
three elevated intervals, and a two-clear-interval reset before window warning.
Window thresholds are 10% some stall or 2% full stall; a one-second interval with
at least 50% full stall is critical. Kernel ten-second averages still warn at
20% some / 5% full and stop critically at 20% full. These are explicit conservative
safety rules, not a calibrated performance-regression threshold. Counter rollback,
missing values and non-finite probes fail closed. Every decision keeps elapsed
window, raw deltas, ratios and policy so transient and sustained signals are visible.
Swap/pageout observations retain their source and units; unsupported Windows
paging counters remain explicitly unavailable.

The existing 8 GiB emergency disk reserve is retained. Initial supervised cold
and graph command caps are 240 seconds, warm 180 seconds, bounded by the same
75-minute shared preparation/sampling deadline and 90-minute job limit. This replaces the
previous universal 120-second cap, which timed out the Windows complete workload
without proving whether compilation was merely slow. Correctness commands keep a separate 180-second ceiling rather than deriving a browser limit from build time. After full successful
bootstrap pairs and confirmed cleanup, this run records observed durations, disk
growth and RSS. Later admission conservatively requires twice the larger observed
root/group RSS where actual available bytes exist, plus observed disk growth over
the reserve. Caps derive from bootstrap time with margin and never exceed their
initial cap or remaining deadline. This is same-run evidence, not a portable
capacity certification or permission to reduce safety floors elsewhere. The job
allows 15 minutes beyond the shared deadline for bounded setup/finalization and
upload; every command still reserves 15 seconds for cleanup and cannot exceed the
remaining shared budget. The 75/90-minute values are hard ceilings, not expected
runtime or a guarantee that every command can consume its individual ceiling.
Current-only performs eight builds plus graph/browser; paired mode performs
sixteen plus a graph/browser for each cohort. Exhaustion stays invalid rather
than dropping samples. Base preparation imports the same deadline function.

Each supervised Node root preloads a small diagnostic module before the CLI or
package manager. Its log contains the actual V8 heap size limit, numeric memory
flags with other argument values omitted, Node's reported constrained memory and
numeric Linux container memory limits when readable. It never prints environment
values, arbitrary eval/script contents or unfiltered arguments. Limits and missing
probes retain their units/status; they are separate from observed process-tree RSS
and never automatically diagnose OOM or raise the heap limit.

Unix builds own a process group. Windows launches suspended into a kill-on-close
Job Object before resuming. Every exit/timeout verifies group/job cleanup; unknown
cleanup stops further sampling and preserves temporary inputs. Kernel root peak
RSS and sampled summed-group RSS are reported separately, not called footprint
or commit. Group RSS may double-count shared pages; live CPU totals are diagnostic,
not formal CPU benchmarks. Phase, cwd, allocated timeout, elapsed time, pressure,
disk, RSS, cleanup and bounded sanitized log tails are retained. No global process
kill is used. Actual OS behavior must be verified by the new remote CI run.
Playwright creates a separate Unix group. Correctness uses public BrowserServer
process identity and an atomic ownership receipt: the supervisor checks its live
parent/group/creation time, includes that group in observations, and verifies its
cleanup separately. An unregistered failed launch is unconfirmed rather than
treated as successful cleanup; no unverified PID is killed based only on JSON.
Unix cleanup records each signal/probe operation, errno, owned PID/group identity
and process state. A single root reaper avoids competing waits. Zombie-only groups
receive a bounded reap/disappearance wait; a permission error is never treated as
proof of exit. A denied signal with live members stops without another termination
API. Cleanup is confirmed only after the known PIDs and group members disappear.

## Farm base/head and history

Base/head require identical fixture, harness, Node/pnpm/Python, OS/architecture,
runner image/version and lockfile. Exact-base native artifacts are independently
validated; head native artifacts are never substituted. Base JS preparation uses
the same supervisor. Incompatible or missing base becomes explicit current-only.
An unconfirmed base process tree prevents head sampling.

Unscored Farm graph checks prove coverage of all 28,673 generated source files,
classify CSS/assets/query/virtual identities and keep retained module counts
separate. The parent trace does not claim to expose the nested native worker
compiler graph. Every enabled cohort, including base in paired mode, runs full
Commerce browser acceptance after measured rounds: first/middle/last category
interaction, stock/discount/shipping, MDX/SVG/styles, original worker import and
WASM type generation. Three interactions do not claim all-category UI coverage.
Page/console/request/HTTP errors invalidate the result. Only local mock data is used.

The Farm-only protocol is distinct from retired mixed-tool and subset protocols;
old reports are rejected rather than relabeled or reused as compatible history.
Current-only needs eight measured/bootstrap builds plus one graph build and one
browser run. Paired mode doubles those counts. The shared 75-minute deadline
and 90-minute job limit remain bounded, not a claim that either budget is
sufficient on every OS. Remote CI must establish completion; deadline exhaustion
stays invalid without reducing workload or the three scored pairs.

History reads at most ten prior trusted runs and distinguishes missing, expired,
incompatible and invalid evidence. Artifacts retain 14 days; history is not permanent
storage. Protocol/harness/fixture/toolchain/image cohorts must match. No cross-OS
absolute ranking is made. Complete Farm sampling may exhaust the current job
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
